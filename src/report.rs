//! Renders findings to stdout and derives the process exit code from them.

use std::process::ExitCode;

use crate::rules::{Finding, Severity};

pub fn print_findings(findings: &[Finding]) {
    for finding in findings {
        println!("{}", finding_line(finding));
    }
    println!("{}", summary_line(findings));
}

fn finding_line(finding: &Finding) -> String {
    let label = severity_label(finding.severity);
    match &finding.page {
        Some(page) => format!("{label} {} {page}: {}", finding.rule_id, finding.message),
        None => format!("{label} {}: {}", finding.rule_id, finding.message),
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "[ERROR]",
        Severity::Warning => "[WARN] ",
    }
}

fn summary_line(findings: &[Finding]) -> String {
    let errors = count(findings, Severity::Error);
    let warnings = count(findings, Severity::Warning);
    format!("{errors} error(s), {warnings} warning(s)")
}

fn count(findings: &[Finding], severity: Severity) -> usize {
    findings.iter().filter(|f| f.severity == severity).count()
}

pub fn exit_code(findings: &[Finding]) -> ExitCode {
    match highest_severity(findings) {
        Some(Severity::Error) => ExitCode::from(1),
        Some(Severity::Warning) => ExitCode::from(2),
        None => ExitCode::SUCCESS,
    }
}

fn highest_severity(findings: &[Finding]) -> Option<Severity> {
    match findings.iter().any(|f| f.severity == Severity::Error) {
        true => Some(Severity::Error),
        false => findings
            .iter()
            .any(|f| f.severity == Severity::Warning)
            .then_some(Severity::Warning),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(rule_id: &'static str) -> Finding {
        Finding {
            rule_id,
            severity: Severity::Error,
            message: "boom".to_string(),
            page: None,
        }
    }

    fn warning(rule_id: &'static str) -> Finding {
        Finding {
            rule_id,
            severity: Severity::Warning,
            message: "meh".to_string(),
            page: None,
        }
    }

    #[test]
    fn no_findings_exits_success() {
        assert_eq!(exit_code(&[]), ExitCode::SUCCESS);
    }

    #[test]
    fn only_warnings_exits_two() {
        assert_eq!(exit_code(&[warning("TGT001")]), ExitCode::from(2));
    }

    #[test]
    fn any_error_exits_one() {
        assert_eq!(
            exit_code(&[warning("TGT001"), error("H57")]),
            ExitCode::from(1)
        );
    }

    #[test]
    fn finding_line_without_page_is_unchanged() {
        assert_eq!(finding_line(&error("H57")), "[ERROR] H57: boom");
    }

    #[test]
    fn finding_line_includes_page_path() {
        let finding = error("H57").on_page("/about".to_string());
        assert_eq!(finding_line(&finding), "[ERROR] H57 /about: boom");
    }

    #[test]
    fn summary_line_counts_each_severity() {
        let findings = vec![error("H57"), error("H42"), warning("TGT001")];
        assert_eq!(summary_line(&findings), "2 error(s), 1 warning(s)");
    }
}
