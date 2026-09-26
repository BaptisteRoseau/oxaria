use std::process::{Command, Output};

fn run(path: &str, extra_args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wcag-checker"))
        .arg(path)
        .args(extra_args)
        .output()
        .expect("failed to run wcag-checker binary")
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
        "H57", "G1", "H42", "G141", "F30", "H44", "F68", "ARIA16", "F84", "H30", "F91", "G87",
        "G152", "F77", "G195",
    ] {
        assert!(report.contains(rule_id), "expected {rule_id} in:\n{report}");
    }
}

#[test]
fn page_with_only_warnings_exits_two() {
    let output = run("tests/assets/warnings_only.html", &[]);
    assert_eq!(exit_code(&output), 2, "stdout:\n{}", stdout(&output));
    let report = stdout(&output);
    assert!(report.contains("G18"), "expected G18 in:\n{report}");
    assert!(report.contains("TGT001"), "expected TGT001 in:\n{report}");
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
