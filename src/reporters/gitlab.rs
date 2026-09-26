//! GitLab Code Quality report, read through `artifacts:reports:codequality`.
//! Format: <https://docs.gitlab.com/ci/testing/code_quality/#code-quality-report-format>

use serde::Serialize;

use super::model::LINE;
use super::{Issue, Report, Reporter};
use crate::rules::Severity;

pub struct GitlabReporter;

#[derive(Serialize)]
struct CodeQualityIssue<'a> {
    description: String,
    check_name: &'a str,
    fingerprint: &'a str,
    severity: &'static str,
    location: CodeQualityLocation<'a>,
}

#[derive(Serialize)]
struct CodeQualityLocation<'a> {
    path: &'a str,
    lines: CodeQualityLines,
}

#[derive(Serialize)]
struct CodeQualityLines {
    begin: u32,
}

impl Reporter for GitlabReporter {
    fn render(&self, report: &Report) -> String {
        let issues: Vec<_> = report.issues.iter().map(code_quality_issue).collect();
        serde_json::to_string_pretty(&issues).expect("plain structs always serialize") + "\n"
    }
}

fn code_quality_issue(issue: &Issue) -> CodeQualityIssue<'_> {
    CodeQualityIssue {
        description: issue.full_text(),
        check_name: issue.rule_id,
        fingerprint: &issue.fingerprint,
        severity: severity(issue.severity),
        location: CodeQualityLocation {
            path: &issue.location.path,
            lines: CodeQualityLines { begin: LINE },
        },
    }
}

fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "major",
        Severity::Warning => "minor",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::reporters::model::tests::{error, on_page, warning};
    use crate::rules::Finding;

    fn render(source: &str, findings: &[Finding]) -> Value {
        let output = GitlabReporter.render(&Report::new(source, findings));
        serde_json::from_str(&output).expect("valid JSON")
    }

    #[test]
    fn empty_report_is_an_empty_array() {
        assert_eq!(render("a.html", &[]), json!([]));
    }

    #[test]
    fn issue_has_every_required_field() {
        let report = Report::new("./a.html", &[error("H57")]);
        let output: Value = serde_json::from_str(&GitlabReporter.render(&report)).unwrap();
        assert_eq!(
            output,
            json!([{
                "description": "boom\nsee: https://www.w3.org/WAI/WCAG22/Techniques/html/H57",
                "check_name": "H57",
                "fingerprint": report.issues[0].fingerprint,
                "severity": "major",
                "location": { "path": "a.html", "lines": { "begin": 1 } },
            }])
        );
    }

    #[test]
    fn help_is_part_of_the_description() {
        let finding = Finding {
            help: Some("add lang".to_string()),
            ..error("FETCH")
        };
        assert_eq!(
            render("a.html", &[finding])[0]["description"],
            "boom\nhelp: add lang"
        );
    }

    #[test]
    fn warnings_are_minor() {
        assert_eq!(render("a.html", &[warning("G18")])[0]["severity"], "minor");
    }

    #[test]
    fn identical_findings_keep_distinct_fingerprints() {
        let output = render("a.html", &[error("F65"), error("F65")]);
        assert_ne!(output[0]["fingerprint"], output[1]["fingerprint"]);
    }

    #[test]
    fn crawled_page_is_located_by_its_url() {
        let output = render(
            "https://example.com",
            &[on_page(error("H57"), "https://example.com/about")],
        );
        assert_eq!(output[0]["location"]["path"], "https://example.com/about");
    }
}
