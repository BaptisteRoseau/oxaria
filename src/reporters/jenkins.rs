//! Jenkins Warnings NG native JSON format, read with
//! `recordIssues(tool: issues(pattern: '<file>'))`.
//! Format: <https://github.com/jenkinsci/warnings-ng-plugin/blob/main/doc/Documentation.md>

use serde::Serialize;

use super::model::LINE;
use super::{Issue, Report, Reporter};
use crate::rules::Severity;

pub struct JenkinsReporter;

#[derive(Serialize)]
struct WarningsReport<'a> {
    issues: Vec<WarningsIssue<'a>>,
    size: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WarningsIssue<'a> {
    file_name: &'a str,
    line_start: u32,
    severity: &'static str,
    category: &'static str,
    #[serde(rename = "type")]
    kind: &'static str,
    message: &'a str,
    fingerprint: &'a str,
}

impl Reporter for JenkinsReporter {
    fn render(&self, report: &Report) -> String {
        let issues: Vec<_> = report.issues.iter().map(warnings_issue).collect();
        let report = WarningsReport {
            size: issues.len(),
            issues,
        };
        serde_json::to_string_pretty(&report).expect("plain structs always serialize") + "\n"
    }
}

fn warnings_issue(issue: &Issue) -> WarningsIssue<'_> {
    WarningsIssue {
        file_name: &issue.location.path,
        line_start: LINE,
        severity: severity(issue.severity),
        category: "WCAG 2.2",
        kind: issue.rule_id,
        message: &issue.message,
        fingerprint: &issue.fingerprint,
    }
}

fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "ERROR",
        Severity::Warning => "NORMAL",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::reporters::model::tests::{error, warning};
    use crate::rules::Finding;

    fn render(source: &str, findings: &[Finding]) -> Value {
        let output = JenkinsReporter.render(&Report::new(source, findings));
        serde_json::from_str(&output).expect("valid JSON")
    }

    #[test]
    fn empty_report_has_no_issues() {
        assert_eq!(render("a.html", &[]), json!({ "issues": [], "size": 0 }));
    }

    #[test]
    fn issue_uses_warnings_ng_field_names() {
        let report = Report::new("a.html", &[error("H57")]);
        let output: Value = serde_json::from_str(&JenkinsReporter.render(&report)).unwrap();
        assert_eq!(
            output,
            json!({
                "issues": [{
                    "fileName": "a.html",
                    "lineStart": 1,
                    "severity": "ERROR",
                    "category": "WCAG 2.2",
                    "type": "H57",
                    "message": "boom",
                    "fingerprint": report.issues[0].fingerprint,
                }],
                "size": 1,
            })
        );
    }

    #[test]
    fn warnings_are_normal_severity() {
        let output = render("a.html", &[error("H57"), warning("G18")]);
        assert_eq!(output["issues"][1]["severity"], "NORMAL");
        assert_eq!(output["size"], 2);
    }
}
