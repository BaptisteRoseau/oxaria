//! JUnit XML, read by Jenkins' `junit` step and GitLab's
//! `artifacts:reports:junit`. One `<testsuite>` per checked page, one
//! `<testcase>` per issue: errors fail, warnings pass with their message in
//! `<system-out>` so JUnit consumers keep the error/warning exit-code split.

use std::collections::{BTreeMap, HashMap};

use super::{Issue, Report, Reporter};
use crate::rules::Severity;

pub struct JunitReporter;

impl Reporter for JunitReporter {
    fn render(&self, report: &Report) -> String {
        let suites: String = issues_by_path(report)
            .into_iter()
            .map(|(path, issues)| testsuite(path, &issues))
            .collect();
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <testsuites name=\"wcag-checker\" tests=\"{}\" failures=\"{}\">\n{suites}</testsuites>\n",
            report.issues.len(),
            report.errors(),
        )
    }
}

/// `BTreeMap` so suites come out in a deterministic order.
fn issues_by_path(report: &Report) -> BTreeMap<&str, Vec<&Issue>> {
    let mut suites: BTreeMap<&str, Vec<&Issue>> = BTreeMap::new();
    for issue in &report.issues {
        suites.entry(&issue.location.path).or_default().push(issue);
    }
    suites
}

fn testsuite(path: &str, issues: &[&Issue]) -> String {
    let failures = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .count();
    let cases: String = issues
        .iter()
        .zip(testcase_names(issues))
        .map(|(issue, name)| testcase(issue, &name))
        .collect();
    format!(
        "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{failures}\">\n{cases}  </testsuite>\n",
        escape(path),
        issues.len(),
    )
}

/// GitLab keys test cases by suite + classname + name and keeps only the
/// last one per key, so the name must be unique within a suite: three
/// unlabeled checkboxes would otherwise count as a single test.
fn testcase_names(issues: &[&Issue]) -> Vec<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    issues
        .iter()
        .map(|issue| {
            let name = format!("{}: {}", issue.rule_id, issue.message);
            let count = seen.entry(name.clone()).or_insert(0);
            *count += 1;
            match *count {
                1 => name,
                n => format!("{name} ({n})"),
            }
        })
        .collect()
}

fn testcase(issue: &Issue, name: &str) -> String {
    let body = match issue.severity {
        Severity::Error => format!(
            "<failure type=\"{}\" message=\"{}\">{}</failure>",
            escape(issue.rule_id),
            escape(&issue.message),
            escape(&issue.message),
        ),
        Severity::Warning => format!("<system-out>{}</system-out>", escape(&issue.message)),
    };
    let path = escape(&issue.location.path);
    format!(
        "    <testcase classname=\"{path}\" name=\"{}\" file=\"{path}\">{body}</testcase>\n",
        escape(name),
    )
}

/// Escapes for both attribute values and text content.
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporters::model::tests::{error, warning};
    use crate::rules::Finding;

    fn render(source: &str, findings: &[Finding]) -> String {
        JunitReporter.render(&Report::new(source, findings))
    }

    #[test]
    fn empty_report_is_an_empty_testsuites() {
        assert_eq!(
            render("a.html", &[]),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <testsuites name=\"wcag-checker\" tests=\"0\" failures=\"0\">\n</testsuites>\n"
        );
    }

    #[test]
    fn error_is_a_failure() {
        let output = render("a.html", &[error("H57")]);
        assert!(output.contains(
            "<testcase classname=\"a.html\" name=\"H57: boom\" file=\"a.html\">\
             <failure type=\"H57\" message=\"boom\">boom</failure></testcase>"
        ));
    }

    #[test]
    fn warning_is_a_passing_testcase() {
        let output = render("a.html", &[error("H57"), warning("G18")]);
        assert!(output.contains("<system-out>meh</system-out>"));
        assert!(output.contains("tests=\"2\" failures=\"1\""));
    }

    #[test]
    fn identical_findings_get_unique_testcase_names() {
        let output = render("a.html", &[error("H44"), error("H44"), error("H44")]);
        assert!(output.contains(r#"name="H44: boom""#), "{output}");
        assert!(output.contains(r#"name="H44: boom (2)""#), "{output}");
        assert!(output.contains(r#"name="H44: boom (3)""#), "{output}");
    }

    #[test]
    fn crawled_pages_get_one_suite_each() {
        let output = render(
            "https://example.com",
            &[
                error("H57").on_page("/b".to_string()),
                error("H42").on_page("/a".to_string()),
                error("H30").on_page("/b".to_string()),
            ],
        );
        let a = output.find("<testsuite name=\"https://example.com/a\" tests=\"1\"");
        let b = output.find("<testsuite name=\"https://example.com/b\" tests=\"2\"");
        assert!(a.is_some() && b.is_some() && a < b, "{output}");
    }

    #[test]
    fn markup_in_messages_is_escaped() {
        let finding = Finding::error("H44", r#"<input name="a&b"> isn't labelled"#.to_string());
        let output = render("a.html", &[finding]);
        assert!(
            output.contains("&lt;input name=&quot;a&amp;b&quot;&gt; isn&apos;t labelled"),
            "{output}"
        );
    }
}
