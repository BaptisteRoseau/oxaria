//! GitHub Actions job summary: GitHub-flavored Markdown appended to the file
//! in `$GITHUB_STEP_SUMMARY`, shown on the workflow run's summary page.
//! Unlike workflow-command annotations (10 errors + 10 warnings per step),
//! a summary has no per-finding cap, only a size limit.
//! Format: <https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#adding-a-job-summary>

use std::cmp::Reverse;

use super::{Issue, Report, Reporter};
use crate::rules::{Severity, Standard};

/// GitHub rejects a step summary over 1 MiB (and then shows none of it). The
/// margin leaves room for whatever else the same step appends to the file.
const MAX_SUMMARY_BYTES: usize = 1_000_000;

const TRUNCATION_NOTE_MARGIN: usize = 200;

const TABLE_HEADER: &str =
    "| Severity | Standard | Rule | Location | Message |\n| --- | --- | --- | --- | --- |\n";

pub struct GithubReporter;

impl Reporter for GithubReporter {
    fn render(&self, report: &Report) -> String {
        let heading = heading(report);
        match report.issues.is_empty() {
            true => format!("{heading}\n"),
            false => format!(
                "{heading}\n\n{TABLE_HEADER}{}",
                rows_within_limit(report, heading.len())
            ),
        }
    }

    /// `$GITHUB_STEP_SUMMARY` is shared by every command of the step, and
    /// GitHub documents appending (`>>`) to it.
    fn appends(&self) -> bool {
        true
    }
}

fn heading(report: &Report) -> String {
    match (report.errors(), report.warnings()) {
        (0, 0) => "## ✅ Accessibility: no issues found".to_string(),
        (0, warnings) => format!("## ⚠️ Accessibility: 0 error(s), {warnings} warning(s)"),
        (errors, warnings) => {
            format!("## ❌ Accessibility: {errors} error(s), {warnings} warning(s)")
        }
    }
}

/// Errors come first, so a summary cut at the size limit keeps the most
/// important findings.
fn rows_within_limit(report: &Report, heading_len: usize) -> String {
    let mut issues: Vec<&Issue> = report.issues.iter().collect();
    issues.sort_by_key(|issue| Reverse(issue.severity == Severity::Error));

    let mut rows = String::new();
    let budget = MAX_SUMMARY_BYTES - TABLE_HEADER.len() - heading_len - TRUNCATION_NOTE_MARGIN;
    for (shown, issue) in issues.iter().enumerate() {
        let row = row(issue);
        if rows.len() + row.len() > budget {
            let hidden = issues.len() - shown;
            rows.push_str(&format!(
                "\n_{hidden} more finding(s) not shown: GitHub limits a job summary to 1 MiB._\n"
            ));
            break;
        }
        rows.push_str(&row);
    }
    rows
}

fn row(issue: &Issue) -> String {
    format!(
        "| {} | {} | {} | {} | {} |\n",
        severity_label(issue.severity),
        issue.standard.map_or("", Standard::name),
        rule_cell(issue),
        escape(&issue.location.path),
        message_cell(issue),
    )
}

fn rule_cell(issue: &Issue) -> String {
    match &issue.reference {
        Some(url) => format!("[`{}`]({url})", issue.rule_id),
        None => format!("`{}`", issue.rule_id),
    }
}

fn message_cell(issue: &Issue) -> String {
    match &issue.help {
        Some(help) => format!("{}<br>💡 {}", escape(&issue.message), escape(help)),
        None => escape(&issue.message),
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "❌ Error",
        Severity::Warning => "⚠️ Warning",
    }
}

/// Messages quote markup (`<img src=...>`), which GitHub would otherwise
/// render as HTML, and may contain `|`, which would split the table cell.
/// CommonMark lets any ASCII punctuation be backslash-escaped.
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '~' | '&' | '!' => {
                escaped.push('\\');
                escaped.push(c);
            }
            '\r' => {}
            '\n' => escaped.push_str("<br>"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporters::model::tests::{error, fatal, on_page, warning};
    use crate::rules::Finding;

    fn render(source: &str, findings: &[Finding]) -> String {
        GithubReporter.render(&Report::new(source, findings))
    }

    #[test]
    fn clean_report_is_a_single_heading() {
        assert_eq!(
            render("a.html", &[]),
            "## ✅ Accessibility: no issues found\n"
        );
    }

    #[test]
    fn findings_are_a_table_under_a_heading() {
        assert_eq!(
            render("./a.html", &[error("H57"), warning("G18")]),
            "## ❌ Accessibility: 1 error(s), 1 warning(s)\n\n\
             | Severity | Standard | Rule | Location | Message |\n\
             | --- | --- | --- | --- | --- |\n\
             | ❌ Error | WCAG 2.2 | [`H57`](https://www.w3.org/WAI/WCAG22/Techniques/html/H57) | a.html | boom |\n\
             | ⚠️ Warning | WCAG 2.2 | [`G18`](https://www.w3.org/WAI/WCAG22/Techniques/general/G18) | a.html | meh |\n"
        );
    }

    #[test]
    fn rules_without_a_reference_are_not_linked() {
        assert!(render("a.html", &[fatal("FETCH")]).contains("| ❌ Error |  | `FETCH` |"));
    }

    #[test]
    fn help_follows_the_message_in_its_cell() {
        let finding = Finding {
            help: Some("add <track>".to_string()),
            ..error("G87")
        };
        assert!(
            render("a.html", &[finding]).contains("| boom<br>💡 add \\<track\\> |"),
            "help missing"
        );
    }

    #[test]
    fn warnings_only_heading() {
        assert!(render("a.html", &[warning("G18")]).starts_with("## ⚠️ Accessibility: 0 error(s)"));
    }

    #[test]
    fn errors_are_listed_before_warnings() {
        let output = render("a.html", &[warning("G18"), error("H57"), warning("TGT001")]);
        let rules: Vec<_> = output
            .lines()
            .skip(4)
            .map(|line| line.split('`').nth(1).unwrap())
            .collect();
        assert_eq!(rules, ["H57", "G18", "TGT001"]);
    }

    #[test]
    fn crawled_pages_are_located_by_url() {
        let output = render(
            "https://example.com",
            &[on_page(error("H57"), "https://example.com/about")],
        );
        assert!(output.contains("| https://example.com/about |"), "{output}");
    }

    #[test]
    fn markup_and_table_syntax_are_escaped() {
        assert_eq!(
            escape("<img src=\"a|b\"> *x* _y_ [l](u) `c` a&b !\\\r\nz"),
            "\\<img src=\"a\\|b\"\\> \\*x\\* \\_y\\_ \\[l\\](u) \\`c\\` a\\&b \\!\\\\<br>z"
        );
    }

    #[test]
    fn summary_stays_under_githubs_size_limit() {
        let long_message = "x".repeat(10_000);
        let findings: Vec<_> = (0..200)
            .map(|_| Finding::error("H57", long_message.clone()))
            .collect();

        let output = render("a.html", &findings);

        assert!(output.len() <= MAX_SUMMARY_BYTES, "{} bytes", output.len());
        assert!(
            output.contains("more finding(s) not shown"),
            "no truncation note"
        );
    }
}
