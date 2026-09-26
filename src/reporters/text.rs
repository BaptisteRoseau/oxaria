//! The human-readable stdout output: one line per issue, followed by
//! rustc-style `= help:`/`= note:` lines, then a summary.

use std::collections::BTreeMap;

use super::{Issue, Report, Reporter};
use crate::rules::{Severity, Standard};

pub struct TextReporter;

impl Reporter for TextReporter {
    fn render(&self, report: &Report) -> String {
        report
            .issues
            .iter()
            .flat_map(issue_lines)
            .chain(std::iter::once(summary_line(report)))
            .map(|line| line + "\n")
            .collect()
    }
}

fn issue_lines(issue: &Issue) -> Vec<String> {
    let help = issue.help.iter().map(|help| format!("  = help: {help}"));
    let note = issue
        .reference
        .iter()
        .map(|url| format!("  = note: see {url}"));
    std::iter::once(issue_line(issue))
        .chain(help)
        .chain(note)
        .collect()
}

fn issue_line(issue: &Issue) -> String {
    let label = severity_label(issue.severity);
    match &issue.location.page {
        Some(page) => format!("{label} {} {page}: {}", issue.rule_label(), issue.message),
        None => format!("{label} {}: {}", issue.rule_label(), issue.message),
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "[ERROR]",
        Severity::Warning => "[WARN] ",
    }
}

fn summary_line(report: &Report) -> String {
    let total = counts(report.errors(), report.warnings());
    match counts_by_standard(report).as_slice() {
        [] => total,
        per_standard => format!("{total} ({})", per_standard.join("; ")),
    }
}

fn counts_by_standard(report: &Report) -> Vec<String> {
    let mut by_standard: BTreeMap<Standard, (usize, usize)> = BTreeMap::new();
    for issue in &report.issues {
        if let Some(standard) = issue.standard {
            let (errors, warnings) = by_standard.entry(standard).or_default();
            match issue.severity {
                Severity::Error => *errors += 1,
                Severity::Warning => *warnings += 1,
            }
        }
    }
    by_standard
        .into_iter()
        .map(|(standard, (errors, warnings))| format!("{standard}: {}", counts(errors, warnings)))
        .collect()
}

fn counts(errors: usize, warnings: usize) -> String {
    format!("{errors} error(s), {warnings} warning(s)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporters::model::tests::{error, fatal, on_page, warning};
    use crate::rules::Finding;

    fn render(source: &str, findings: &[Finding]) -> String {
        TextReporter.render(&Report::new(source, findings))
    }

    #[test]
    fn issue_line_without_page_is_unchanged() {
        assert_eq!(
            render("a.html", &[fatal("FETCH")]),
            "[ERROR] FETCH: boom\n1 error(s), 0 warning(s)\n"
        );
    }

    #[test]
    fn help_and_note_follow_the_issue_line() {
        let finding = Finding {
            help: Some("add lang=\"en\"".to_string()),
            ..error("H57")
        };
        assert_eq!(
            render("a.html", &[finding]),
            "[ERROR] WCAG 2.2 H57: boom\n  \
             = help: add lang=\"en\"\n  \
             = note: see https://www.w3.org/WAI/WCAG22/Techniques/html/H57\n\
             1 error(s), 0 warning(s) (WCAG 2.2: 1 error(s), 0 warning(s))\n"
        );
    }

    #[test]
    fn issue_line_includes_page_path() {
        let report = render(
            "https://example.com",
            &[on_page(error("H57"), "https://example.com/about")],
        );
        assert!(
            report.starts_with("[ERROR] WCAG 2.2 H57 /about: boom\n"),
            "{report}"
        );
    }

    #[test]
    fn warnings_are_padded_to_align_with_errors() {
        assert!(render("a.html", &[warning("G18")]).starts_with("[WARN]  WCAG 2.2 G18: meh\n"));
    }

    #[test]
    fn summary_line_counts_each_severity() {
        let report = render("a.html", &[error("H57"), error("H42"), warning("TGT001")]);
        assert!(
            report.ends_with("2 error(s), 1 warning(s) (WCAG 2.2: 2 error(s), 1 warning(s))\n"),
            "{report}"
        );
    }

    #[test]
    fn summary_line_breaks_counts_down_by_standard() {
        let findings = [
            fatal("FETCH"),
            warning("G18"),
            fatal("2779a5").in_standard(Standard::Act),
        ];
        assert!(render("a.html", &findings).ends_with(
            "2 error(s), 1 warning(s) \
                 (WCAG 2.2: 0 error(s), 1 warning(s); ACT: 1 error(s), 0 warning(s))\n"
        ));
    }

    #[test]
    fn empty_report_prints_only_the_summary() {
        assert_eq!(render("a.html", &[]), "0 error(s), 0 warning(s)\n");
    }
}
