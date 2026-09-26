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
    check_name: String,
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
        description: description(issue),
        check_name: issue.rule_label(),
        fingerprint: &issue.fingerprint,
        severity: severity(issue.severity),
        location: CodeQualityLocation {
            path: &issue.location.path,
            lines: CodeQualityLines { begin: LINE },
        },
    }
}

fn description(issue: &Issue) -> String {
    format!("{}: {}", issue.rule_label(), issue.full_text())
        .lines()
        .map(code_span_markup)
        .collect::<Vec<_>>()
        .join("  \n")
}

fn code_span_markup(line: &str) -> String {
    let mut spanned = String::with_capacity(line.len());
    let mut rest = line;
    while let Some((before, tag, after)) = next_tag(rest) {
        spanned.push_str(before);
        spanned.push('`');
        spanned.push_str(tag);
        spanned.push('`');
        rest = after;
    }
    spanned.push_str(rest);
    spanned
}

fn next_tag(text: &str) -> Option<(&str, &str, &str)> {
    let start = text.find('<')?;
    let end = start + text[start..].find('>')? + 1;
    Some((&text[..start], &text[start..end], &text[end..]))
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
    use crate::reporters::model::tests::{error, fatal, on_page, warning};
    use crate::rules::{Finding, Standard};

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
                "description": "WCAG 2.2 H57: boom  \nsee: https://www.w3.org/WAI/WCAG22/Techniques/html/H57",
                "check_name": "WCAG 2.2 H57",
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
            ..fatal("FETCH")
        };
        assert_eq!(
            render("a.html", &[finding])[0]["description"],
            "FETCH: boom  \nhelp: add lang"
        );
    }

    #[test]
    fn quoted_markup_becomes_code_spans() {
        let finding = Finding {
            help: Some(r#"add <label for="a">...</label>"#.to_string()),
            ..Finding::error("G87", "<video> has no track (at body > video)".to_string())
                .in_standard(Standard::Wcag22)
        };
        assert_eq!(
            render("a.html", &[finding])[0]["description"],
            "WCAG 2.2 G87: `<video>` has no track (at body > video)  \n\
             help: add `<label for=\"a\">`...`</label>`  \n\
             see: https://www.w3.org/WAI/WCAG22/Techniques/general/G87"
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
