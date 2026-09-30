use std::process::{Command, Output};

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// `--report-github` defaults to `$GITHUB_STEP_SUMMARY`: without clearing it,
/// every test run on GitHub Actions would append to the job summary.
fn run(path: &str, extra_args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oxaria"))
        .arg(path)
        .args(extra_args)
        .env_remove("GITHUB_STEP_SUMMARY")
        .output()
        .expect("failed to run oxaria binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn exit_code(output: &Output) -> i32 {
    output.status.code().expect("process terminated by signal")
}

#[test]
fn clean_page_exits_zero_with_no_findings() {
    let output = run("tests/assets/clean.html", &[]);
    assert_eq!(exit_code(&output), 0, "stdout:\n{}", stdout(&output));
    assert!(stdout(&output).contains("0 error(s), 0 warning(s)"));
}

#[test]
fn page_with_violations_exits_one() {
    let output = run("tests/assets/errors.html", &[]);
    assert_eq!(exit_code(&output), 1, "stdout:\n{}", stdout(&output));
    let report = stdout(&output);
    for rule_id in [
        "H57", "G1", "H42", "G141", "F30", "H44", "F68", "ARIA1", "LNK001", "H30", "F91", "G87",
        "G186", "IDS001", "G195", "F93",
    ] {
        assert!(
            report.contains(&format!("[ERROR] WCAG 2.2 {rule_id}: ")),
            "expected {rule_id} in:\n{report}"
        );
    }
    for finding in [
        "WAI-ARIA 1.2 ARIA-IDREF001",
        "ACT b5c3f8",
        "ACT 97a4e1",
        "ACT e086e5",
    ] {
        assert!(
            report.contains(&format!("[ERROR] {finding}: ")),
            "expected {finding} in:\n{report}"
        );
    }
    assert!(
        report.contains("26 error(s), 0 warning(s) (WCAG 2.2: 20 error(s), 0 warning(s); WAI-ARIA 1.2: 1 error(s), 0 warning(s); ACT: 5 error(s), 0 warning(s))"),
        "{report}"
    );
}

#[test]
fn findings_name_the_element_they_are_about() {
    let report = stdout(&run("tests/assets/errors.html", &[]));
    assert!(
        report.contains(
            "[ERROR] WCAG 2.2 H44: <input type=\"email\"> has no associated label (at input#signup-email)"
        ),
        "{report}"
    );
    assert!(
        report.contains("[ERROR] WCAG 2.2 H44: <input type=\"checkbox\"> has no associated label (at body > input:nth-of-type(3))"),
        "{report}"
    );
    assert!(
        report.contains("[ERROR] WCAG 2.2 H42: page has no <h1> element\n"),
        "{report}"
    );
}

#[test]
fn page_with_only_warnings_exits_two() {
    let output = run("tests/assets/warnings_only.html", &[]);
    assert_eq!(exit_code(&output), 2, "stdout:\n{}", stdout(&output));
    let report = stdout(&output);
    assert!(report.contains("G18"), "expected G18 in:\n{report}");
    assert!(report.contains("TGT001"), "expected TGT001 in:\n{report}");
    assert!(
        report.contains("[WARN]  ACT afw4f7: <p> text #999999 on #ffffff"),
        "expected afw4f7 in:\n{report}"
    );
    assert!(
        !report.contains("[ERROR]"),
        "unexpected error in:\n{report}"
    );
}

#[test]
fn raising_target_size_threshold_flags_an_otherwise_clean_page() {
    let output = run(
        "tests/assets/clean.html",
        &["--target-size-threshold", "9999"],
    );
    let report = stdout(&output);
    assert!(report.contains("TGT001"), "expected TGT001 in:\n{report}");
    assert_eq!(exit_code(&output), 2, "stdout:\n{report}");
}

#[test]
fn lowering_contrast_threshold_clears_the_warning() {
    let output = run(
        "tests/assets/warnings_only.html",
        &["--contrast-threshold", "1.0"],
    );
    let report = stdout(&output);
    assert!(!report.contains("G18"), "expected no G18 in:\n{report}");
    assert!(
        !report.contains("afw4f7"),
        "expected no afw4f7 in:\n{report}"
    );
    assert!(report.contains("TGT001"), "expected TGT001 in:\n{report}");
    assert_eq!(exit_code(&output), 2, "stdout:\n{report}");
}

#[test]
fn lowering_target_size_threshold_clears_that_warning_too() {
    let output = run(
        "tests/assets/warnings_only.html",
        &[
            "--contrast-threshold",
            "1.0",
            "--target-size-threshold",
            "1",
        ],
    );
    assert_eq!(exit_code(&output), 0, "stdout:\n{}", stdout(&output));
}

#[test]
fn missing_file_exits_one() {
    let output = run("tests/assets/does-not-exist.html", &[]);
    assert_eq!(exit_code(&output), 1, "stdout:\n{}", stdout(&output));
}

fn fixture(name: &str) -> ResponseTemplate {
    let body = std::fs::read_to_string(format!("tests/assets/{name}")).unwrap();
    ResponseTemplate::new(200).set_body_raw(body, "text/html; charset=utf-8")
}

async fn serve(server: &MockServer, route: &str, response: ResponseTemplate) {
    Mock::given(method("GET"))
        .and(path(route))
        .respond_with(response)
        .mount(server)
        .await;
}

/// `/` (clean) links to `/products` (warnings only), `/contact` (errors),
/// and a PDF. Every other path -- e.g. `/privacy-policy`, `/products/1` --
/// is unmatched and gets wiremock's default `404`.
async fn fixture_site() -> MockServer {
    let server = MockServer::start().await;
    serve(&server, "/", fixture("clean.html")).await;
    serve(&server, "/products", fixture("warnings_only.html")).await;
    serve(&server, "/contact", fixture("errors.html")).await;
    let pdf = ResponseTemplate::new(200).set_body_raw("%PDF-1.7", "application/pdf");
    serve(&server, "/annual-report-2026.pdf", pdf).await;
    server
}

/// The binary blocks while the mock server needs the runtime to answer it.
async fn run_async(url: String, extra_args: &[&str]) -> Output {
    let extra_args: Vec<String> = extra_args.iter().map(|arg| arg.to_string()).collect();
    tokio::task::spawn_blocking(move || {
        let extra_args: Vec<&str> = extra_args.iter().map(String::as_str).collect();
        run(&url, &extra_args)
    })
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn full_site_scan_reports_findings_and_http_errors_per_page() {
    let server = fixture_site().await;
    let output = run_async(format!("{}/", server.uri()), &["--full-site-scan"]).await;
    let report = stdout(&output);

    assert_eq!(exit_code(&output), 1, "stdout:\n{report}");
    for expected in [
        "G18 /products:",
        "H57 /contact:",
        "HTTP /privacy-policy: HTTP 404",
        "HTTP /products/1: HTTP 404",
    ] {
        assert!(
            report.contains(expected),
            "expected {expected} in:\n{report}"
        );
    }
    assert!(
        !report.contains("annual-report"),
        "PDF was reported:\n{report}"
    );
    assert!(
        !report.contains(" /: "),
        "clean home page was reported:\n{report}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn full_site_scan_max_pages_stops_the_crawl() {
    let server = fixture_site().await;
    let output = run_async(
        format!("{}/", server.uri()),
        &["--full-site-scan", "--full-site-scan-max-pages", "1"],
    )
    .await;

    assert_eq!(exit_code(&output), 0, "stdout:\n{}", stdout(&output));
    assert!(stdout(&output).contains("0 error(s), 0 warning(s)"));
}

#[test]
fn full_site_scan_on_a_local_file_scans_just_that_file() {
    let output = run("tests/assets/clean.html", &["--full-site-scan"]);
    assert_eq!(exit_code(&output), 0, "stdout:\n{}", stdout(&output));
    assert!(stdout(&output).contains("0 error(s), 0 warning(s)"));
}

#[test]
fn standards_restricts_the_rules_that_run() {
    let output = run("tests/assets/errors.html", &["--standards", "act,aria1.2"]);
    let out = stdout(&output);
    assert!(out.contains("ACT:"), "stdout:\n{out}");
    assert!(out.contains("WAI-ARIA 1.2:"), "stdout:\n{out}");
    assert!(!out.contains("WCAG 2.2:"), "stdout:\n{out}");
}

#[test]
fn list_rules_prints_the_selected_standards_rules_without_a_page() {
    let output = run("--list-rules", &["--standards", "act,wcag2.2"]);
    assert_eq!(exit_code(&output), 0);
    let out = stdout(&output);
    assert!(
        out.lines()
            .any(|line| line == "ACT 2779a5: Give every page a non-empty title element"),
        "stdout:\n{out}"
    );
    assert!(
        out.lines()
            .any(|line| line == "WCAG 2.2 H25: Give every page a non-empty title element"),
        "stdout:\n{out}"
    );
    assert!(!out.contains("WAI-ARIA 1.2"), "stdout:\n{out}");
}

#[rstest::rstest]
#[case("")]
#[case(",")]
#[case("wcag2.2,,act")]
#[case("wcag")]
fn standards_rejects_empty_or_unknown_values(#[case] value: &str) {
    let output = run("tests/assets/clean.html", &["--standards", value]);
    assert_ne!(exit_code(&output), 0);
    assert!(String::from_utf8_lossy(&output.stderr).contains("--standards"));
}

#[test]
fn max_pages_requires_full_site_scan() {
    let output = run(
        "tests/assets/clean.html",
        &["--full-site-scan-max-pages", "3"],
    );
    assert_ne!(exit_code(&output), 0);
    assert!(String::from_utf8_lossy(&output.stderr).contains("--full-site-scan"));
}

fn read(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn json(path: &std::path::Path) -> serde_json::Value {
    serde_json::from_str(&read(path)).expect("report is valid JSON")
}

#[test]
fn reports_are_written_alongside_the_usual_output() {
    let dir = tempfile::tempdir().unwrap();
    let gitlab = dir.path().join("gl-code-quality.json");
    let jenkins = dir.path().join("reports/warnings-ng.json");
    let junit = dir.path().join("junit.xml");
    let output = run(
        "tests/assets/errors.html",
        &[
            "--report-gitlab",
            gitlab.to_str().unwrap(),
            "--report-jenkins",
            jenkins.to_str().unwrap(),
            "--report-junit",
            junit.to_str().unwrap(),
        ],
    );

    assert_eq!(exit_code(&output), 1, "stdout:\n{}", stdout(&output));
    assert!(stdout(&output).contains("[ERROR] WCAG 2.2 H57"));

    let gitlab = json(&gitlab);
    let issues = gitlab.as_array().expect("GitLab report is an array");
    assert!(issues.iter().any(|i| i["check_name"] == "WCAG 2.2 H57"));
    assert!(
        issues
            .iter()
            .all(|i| i["location"]["path"] == "tests/assets/errors.html"
                && i["location"]["lines"]["begin"] == 1)
    );

    let jenkins = json(&jenkins);
    assert_eq!(jenkins["size"], issues.len());
    assert!(jenkins["issues"][0]["fileName"] == "tests/assets/errors.html");

    let junit = read(&junit);
    assert!(junit.starts_with("<?xml"));
    assert!(junit.contains(r#"name="WCAG 2.2 H57: "#), "{junit}");
}

#[test]
fn github_summary_can_go_to_stdout() {
    let output = run(
        "tests/assets/warnings_only.html",
        &["--quiet", "--report-github", "-"],
    );
    let report = stdout(&output);

    assert_eq!(exit_code(&output), 2, "stdout:\n{report}");
    assert!(
        report.starts_with("## ⚠️ Accessibility: 0 error(s), 4 warning(s)\n"),
        "{report}"
    );
    assert_eq!(report.matches("| ⚠️ Warning |").count(), 4, "{report}");
}

#[test]
fn github_summary_defaults_to_github_step_summary_and_appends() {
    let dir = tempfile::tempdir().unwrap();
    let summary = dir.path().join("step_summary.md");
    std::fs::write(&summary, "previous command\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_oxaria"))
        .args(["tests/assets/errors.html", "-q"])
        .env("GITHUB_STEP_SUMMARY", &summary)
        .output()
        .unwrap();

    assert_eq!(exit_code(&output), 1);
    let summary = read(&summary);
    assert!(
        summary.starts_with("previous command\n## ❌ Accessibility: 26 error(s)"),
        "{summary}"
    );
    assert_eq!(summary.matches("| ❌ Error |").count(), 26, "{summary}");
}

#[test]
fn empty_github_step_summary_writes_no_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_oxaria"))
        .args(["tests/assets/clean.html", "-q"])
        .env("GITHUB_STEP_SUMMARY", "")
        .output()
        .unwrap();
    assert_eq!(exit_code(&output), 0);
}

#[test]
fn quiet_prints_nothing_but_keeps_the_exit_code() {
    let output = run("tests/assets/errors.html", &["-q"]);
    assert_eq!(exit_code(&output), 1);
    assert_eq!(stdout(&output), "");
}

#[test]
fn quiet_and_verbose_are_exclusive() {
    let output = run("tests/assets/clean.html", &["-q", "-v"]);
    assert_ne!(exit_code(&output), 0);
    assert!(String::from_utf8_lossy(&output.stderr).contains("--quiet"));
}

#[test]
fn conflicting_report_destinations_fail_before_checking() {
    let dir = tempfile::tempdir().unwrap();
    let report = dir.path().join("report.json");
    let same_report = dir.path().join("sub/../report.json");
    let output = run(
        "tests/assets/errors.html",
        &[
            "--report-gitlab",
            report.to_str().unwrap(),
            "--report-jenkins",
            same_report.to_str().unwrap(),
        ],
    );
    let log = stdout(&output);

    assert_eq!(exit_code(&output), 1, "stdout:\n{log}");
    assert!(
        log.contains("--report-gitlab and --report-jenkins both write to"),
        "{log}"
    );
    assert!(
        !log.contains("error(s)"),
        "the check should not run:\n{log}"
    );
    assert!(!report.exists());
}

#[test]
fn a_fatal_error_is_still_reported() {
    let dir = tempfile::tempdir().unwrap();
    let gitlab = dir.path().join("gl.json");
    let output = run(
        "tests/assets/does-not-exist.html",
        &["--report-gitlab", gitlab.to_str().unwrap()],
    );

    assert_eq!(exit_code(&output), 1, "stdout:\n{}", stdout(&output));
    let issues = json(&gitlab);
    assert_eq!(issues.as_array().unwrap().len(), 1);
    assert_eq!(issues[0]["check_name"], "INPUT");
    assert_eq!(issues[0]["severity"], "major");
}

#[test]
fn aria_page_reports_each_new_standard_with_its_spec_link() {
    let output = run("tests/assets/aria.html", &[]);
    let report = stdout(&output);
    assert_eq!(exit_code(&output), 1, "stdout:\n{report}");
    for expected in [
        "[ERROR] WAI-ARIA 1.2 ARIA-ROLE002: role=\"widget\" is an abstract role",
        "note: see https://www.w3.org/TR/wai-aria-1.2/#isAbstract",
        "[ERROR] WAI-ARIA 1.2 ARIA-DEPR001: <div> uses the deprecated role=\"directory\"",
        "[ERROR] ARIA in HTML HTMLARIA014: <input type=\"text\" required aria-required=\"true\">",
        "note: see https://www.w3.org/TR/html-aria/#docconformance-attr",
        "[ERROR] ARIA in HTML HTMLARIA010: <p hidden aria-hidden=\"true\">",
        "[ERROR] ACT 674b10: <div role=\"lnik\"> contains no valid WAI-ARIA role (at main#main-content > div:nth-of-type(3))",
        "note: see https://www.w3.org/WAI/standards-guidelines/act/rules/674b10/",
        "[ERROR] ACT 6a7281: <button aria-expanded=\"collapsed\">",
        "11 error(s), 0 warning(s) (WAI-ARIA 1.2: 3 error(s), 0 warning(s); ARIA in HTML: 5 error(s), 0 warning(s); ACT: 3 error(s), 0 warning(s))",
    ] {
        assert!(
            report.contains(expected),
            "expected {expected} in:\n{report}"
        );
    }
    assert!(!report.contains("WCAG 2.2"), "{report}");
}

#[test]
fn ci_reports_name_the_new_standards() {
    let dir = tempfile::tempdir().unwrap();
    let gitlab = dir.path().join("gl-code-quality.json");
    let junit = dir.path().join("junit.xml");
    let output = run(
        "tests/assets/aria.html",
        &[
            "--report-gitlab",
            gitlab.to_str().unwrap(),
            "--report-junit",
            junit.to_str().unwrap(),
        ],
    );
    assert_eq!(exit_code(&output), 1, "stdout:\n{}", stdout(&output));

    let gitlab = json(&gitlab);
    let check_names: Vec<_> = gitlab
        .as_array()
        .expect("GitLab report is an array")
        .iter()
        .map(|issue| issue["check_name"].as_str().unwrap().to_string())
        .collect();
    for expected in [
        "WAI-ARIA 1.2 ARIA-ROLE002",
        "ARIA in HTML HTMLARIA002",
        "ACT 674b10",
    ] {
        assert!(
            check_names.iter().any(|name| name == expected),
            "{check_names:?}"
        );
    }

    let junit = read(&junit);
    for expected in [
        r#"name="WAI-ARIA 1.2 ARIA-VAL001: "#,
        r#"name="ARIA in HTML HTMLARIA015: "#,
        r#"name="ACT 6a7281: "#,
    ] {
        assert!(junit.contains(expected), "expected {expected} in:\n{junit}");
    }
}
