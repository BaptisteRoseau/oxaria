//! GitHub Actions workflow commands (`::error ...::message`), which GitHub
//! turns into annotations on the run and the pull request -- but only when
//! they are printed to a step's stdout, hence `--report-github -`.
//! Format: <https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands>

use super::model::LINE;
use super::{Issue, Report, Reporter};
use crate::rules::Severity;

pub struct GithubReporter;

impl Reporter for GithubReporter {
    fn render(&self, report: &Report) -> String {
        report
            .issues
            .iter()
            .map(|issue| command(issue) + "\n")
            .collect()
    }
}

fn command(issue: &Issue) -> String {
    format!(
        "::{} file={},line={LINE},title={}::{}",
        command_name(issue.severity),
        escape_property(&issue.location.path),
        escape_property(&title(issue)),
        escape_data(&issue.message),
    )
}

fn command_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

fn title(issue: &Issue) -> String {
    match &issue.location.page {
        Some(page) => format!("WCAG {} ({page})", issue.rule_id),
        None => format!("WCAG {}", issue.rule_id),
    }
}

/// Same escaping as `escapeData` in `actions/toolkit`'s `command.ts`.
fn escape_data(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// Same escaping as `escapeProperty` in `actions/toolkit`'s `command.ts`.
fn escape_property(value: &str) -> String {
    escape_data(value).replace(':', "%3A").replace(',', "%2C")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporters::model::tests::{error, warning};
    use crate::rules::Finding;

    fn render(source: &str, findings: &[Finding]) -> String {
        GithubReporter.render(&Report::new(source, findings))
    }

    #[test]
    fn empty_report_prints_nothing() {
        assert_eq!(render("a.html", &[]), "");
    }

    #[test]
    fn error_becomes_an_error_command() {
        assert_eq!(
            render("./a.html", &[error("H57")]),
            "::error file=a.html,line=1,title=WCAG H57::boom\n"
        );
    }

    #[test]
    fn warning_becomes_a_warning_command() {
        assert!(render("a.html", &[warning("G18")]).starts_with("::warning "));
    }

    #[test]
    fn crawled_page_url_and_title_are_escaped() {
        let output = render(
            "https://example.com",
            &[error("H57").on_page("/about".to_string())],
        );
        assert_eq!(
            output,
            "::error file=https%3A//example.com/about,line=1,title=WCAG H57 (/about)::boom\n"
        );
    }

    #[test]
    fn message_escapes_percent_and_newlines_but_not_colons() {
        assert_eq!(escape_data("50%\r\na: b, c"), "50%25%0D%0Aa: b, c");
    }

    #[test]
    fn property_escapes_colons_and_commas() {
        assert_eq!(escape_property("a:b,c%\n"), "a%3Ab%2Cc%25%0A");
    }

    #[test]
    fn each_issue_is_on_its_own_line() {
        let output = render("a.html", &[error("H57"), warning("G18")]);
        assert_eq!(output.lines().count(), 2);
    }
}
