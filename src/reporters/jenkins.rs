//! Jenkins Warnings NG native JSON format, read with
//! `recordIssues(tool: issues(pattern: '<file>'))`.
//! Format: <https://github.com/jenkinsci/warnings-ng-plugin/blob/main/doc/Documentation.md>

use serde::Serialize;

use super::junit::escape;
use super::model::LINE;
use super::{Issue, Report, Reporter};
use crate::rules::{Severity, Standard};

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
    /// HTML, shown in the issue's details.
    description: String,
    fingerprint: &'a str,
    /// Warnings NG drops issues it considers equal, and its equality ignores
    /// `fingerprint`: two unlabeled checkboxes (same rule, file, line, and
    /// message) would collapse into one. This free-form field does count.
    additional_properties: &'a str,
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
        category: issue.standard.map_or("wcag-checker", Standard::name),
        kind: issue.rule_id,
        message: &issue.message,
        description: description(issue),
        fingerprint: &issue.fingerprint,
        additional_properties: &issue.fingerprint,
    }
}

fn description(issue: &Issue) -> String {
    let help = issue
        .help
        .iter()
        .map(|help| format!("<p>help: {}</p>", escape(help)));
    let reference = issue.reference.iter().map(|url| {
        let url = escape(url);
        format!("<p>see: <a href=\"{url}\">{url}</a></p>")
    });
    help.chain(reference).collect()
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
    use crate::reporters::model::tests::{error, fatal, warning};
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
                    "description": "<p>see: <a href=\"https://www.w3.org/WAI/WCAG22/Techniques/html/H57\">https://www.w3.org/WAI/WCAG22/Techniques/html/H57</a></p>",
                    "fingerprint": report.issues[0].fingerprint,
                    "additionalProperties": report.issues[0].fingerprint,
                }],
                "size": 1,
            })
        );
    }

    #[test]
    fn identical_findings_stay_distinct_for_warnings_ng() {
        let output = render("a.html", &[error("H44"), error("H44")]);
        assert_ne!(
            output["issues"][0]["additionalProperties"],
            output["issues"][1]["additionalProperties"]
        );
    }

    #[test]
    fn help_is_escaped_into_the_description() {
        let finding = Finding {
            help: Some("add <track>".to_string()),
            ..fatal("FETCH")
        };
        assert_eq!(
            render("a.html", &[finding])["issues"][0]["description"],
            "<p>help: add &lt;track&gt;</p>"
        );
    }

    #[test]
    fn checker_failures_have_their_own_category() {
        assert_eq!(
            render("a.html", &[fatal("FETCH")])["issues"][0]["category"],
            "wcag-checker"
        );
    }

    #[test]
    fn warnings_are_normal_severity() {
        let output = render("a.html", &[error("H57"), warning("G18")]);
        assert_eq!(output["issues"][1]["severity"], "NORMAL");
        assert_eq!(output["size"], 2);
    }
}
