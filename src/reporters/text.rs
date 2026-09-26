//! The human-readable stdout output: one line per issue, then a summary.

use super::{Issue, Report, Reporter};
use crate::rules::Severity;

pub struct TextReporter;

impl Reporter for TextReporter {
    fn render(&self, report: &Report) -> String {
        report
            .issues
            .iter()
            .map(issue_line)
            .chain(std::iter::once(summary_line(report)))
            .map(|line| line + "\n")
            .collect()
    }
}

fn issue_line(issue: &Issue) -> String {
    let label = severity_label(issue.severity);
    match &issue.location.page {
        Some(page) => format!("{label} {} {page}: {}", issue.rule_id, issue.message),
        None => format!("{label} {}: {}", issue.rule_id, issue.message),
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "[ERROR]",
        Severity::Warning => "[WARN] ",
    }
}

fn summary_line(report: &Report) -> String {
    format!(
        "{} error(s), {} warning(s)",
        report.errors(),
        report.warnings()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporters::model::tests::{error, on_page, warning};

    fn render(source: &str, findings: &[crate::rules::Finding]) -> String {
        TextReporter.render(&Report::new(source, findings))
    }

    #[test]
    fn issue_line_without_page_is_unchanged() {
        assert_eq!(
            render("a.html", &[error("H57")]),
            "[ERROR] H57: boom\n1 error(s), 0 warning(s)\n"
        );
    }

    #[test]
    fn issue_line_includes_page_path() {
        let report = render(
            "https://example.com",
            &[on_page(error("H57"), "https://example.com/about")],
        );
        assert!(report.starts_with("[ERROR] H57 /about: boom\n"), "{report}");
    }

    #[test]
    fn warnings_are_padded_to_align_with_errors() {
        assert!(render("a.html", &[warning("G18")]).starts_with("[WARN]  G18: meh\n"));
    }

    #[test]
    fn summary_line_counts_each_severity() {
        let report = render("a.html", &[error("H57"), error("H42"), warning("TGT001")]);
        assert!(report.ends_with("2 error(s), 1 warning(s)\n"), "{report}");
    }

    #[test]
    fn empty_report_prints_only_the_summary() {
        assert_eq!(render("a.html", &[]), "0 error(s), 0 warning(s)\n");
    }
}
