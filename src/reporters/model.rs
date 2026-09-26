//! The format-independent view of a run that every [`Reporter`] renders:
//! each [`Finding`] enriched with where it was found and a stable
//! fingerprint, plus the exit code derived from them.
//!
//! [`Reporter`]: super::Reporter

use std::collections::HashMap;
use std::process::ExitCode;

use url::Url;

use crate::page;
use crate::rules::{Finding, Severity, Standard};

/// Every CI format requires a line number, but litehtml exposes no source
/// positions, so every issue points at the first line of its document.
pub const LINE: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// The local file as given on the command line (minus a leading `./`,
    /// which GitLab can't link), or the URL of the page the issue is on.
    pub path: String,
    /// URL path of the page, during a full-site scan only.
    pub page: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub rule_id: &'static str,
    /// `None` for failures of the checker itself (`FETCH`, `HTTP`, ...).
    pub standard: Option<Standard>,
    pub severity: Severity,
    /// The finding's message, followed by the element it is about (if any):
    /// no CI format has a field for it, and without it identical findings
    /// (a page's many 19px buttons) can't be told apart.
    pub message: String,
    /// Kept out of `message`, which the fingerprint and JUnit test names are
    /// built from: rewording a hint must not turn every issue into a new one.
    pub help: Option<String>,
    /// The W3C page documenting the rule.
    pub reference: Option<String>,
    pub location: Location,
    pub fingerprint: String,
}

impl Issue {
    /// The rule ID, preceded by its standard's name when it has one.
    pub fn rule_label(&self) -> String {
        match self.standard {
            Some(standard) => format!("{standard} {}", self.rule_id),
            None => self.rule_id.to_string(),
        }
    }

    /// The message followed by the help and reference lines, for formats
    /// with a single free-text field.
    pub fn full_text(&self) -> String {
        let help = self.help.iter().map(|help| format!("help: {help}"));
        let reference = self.reference.iter().map(|url| format!("see: {url}"));
        std::iter::once(self.message.clone())
            .chain(help)
            .chain(reference)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub issues: Vec<Issue>,
}

impl Report {
    pub fn new(source: &str, findings: &[Finding]) -> Self {
        let mut occurrences = HashMap::new();
        let issues = findings
            .iter()
            .map(|finding| {
                let location = location(source, finding.page.as_ref());
                let message = message(finding);
                let key = fingerprint_key(finding, &location, &message);
                let occurrence = occurrences.entry(key.clone()).or_insert(0usize);
                *occurrence += 1;
                Issue {
                    rule_id: finding.rule_id,
                    standard: finding.standard,
                    severity: finding.severity,
                    message,
                    help: finding.help.clone(),
                    reference: finding
                        .standard
                        .and_then(|standard| standard.reference_url(finding.rule_id)),
                    location,
                    fingerprint: fingerprint(&format!("{key}\u{1f}{occurrence}")),
                }
            })
            .collect();
        Report { issues }
    }

    pub fn errors(&self) -> usize {
        self.count(Severity::Error)
    }

    pub fn warnings(&self) -> usize {
        self.count(Severity::Warning)
    }

    fn count(&self, severity: Severity) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
    }

    pub fn exit_code(&self) -> ExitCode {
        match (self.errors(), self.warnings()) {
            (0, 0) => ExitCode::SUCCESS,
            (0, _) => ExitCode::from(2),
            _ => ExitCode::from(1),
        }
    }
}

/// A crawled page is located by the URL it was served from, which may be on
/// another host than the source when the start URL redirects.
fn location(source: &str, page: Option<&Url>) -> Location {
    let path = match (page, page::is_url(source)) {
        (Some(page), _) => page.to_string(),
        (None, true) => source.to_string(),
        (None, false) => strip_current_dir(source).to_string(),
    };
    Location {
        path,
        page: page.map(|page| page.path().to_string()),
    }
}

fn message(finding: &Finding) -> String {
    match &finding.element {
        Some(element) => format!("{} (at {element})", finding.message),
        None => finding.message.clone(),
    }
}

fn strip_current_dir(mut path: &str) -> &str {
    while let Some(rest) = path.strip_prefix("./") {
        path = rest;
    }
    path
}

fn fingerprint_key(finding: &Finding, location: &Location, message: &str) -> String {
    [
        finding.standard.map_or("", Standard::name),
        finding.rule_id,
        &location.path,
        location.page.as_deref().unwrap_or(""),
        message,
    ]
    .join("\u{1f}")
}

/// FNV-1a 64: std's `DefaultHasher` isn't guaranteed stable across Rust
/// releases, and GitLab compares fingerprints between pipelines to tell new
/// issues from fixed ones.
fn fingerprint(key: &str) -> String {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let hash = key.bytes().fold(OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(PRIME)
    });
    format!("{hash:016x}")
}

#[cfg(test)]
pub(crate) mod tests {
    use rstest::rstest;

    use super::*;

    pub(crate) fn error(rule_id: &'static str) -> Finding {
        fatal(rule_id).in_standard(Standard::Wcag22)
    }

    pub(crate) fn fatal(rule_id: &'static str) -> Finding {
        Finding::error(rule_id, "boom".to_string())
    }

    pub(crate) fn on_page(finding: Finding, url: &str) -> Finding {
        finding.on_page(&Url::parse(url).unwrap())
    }

    pub(crate) fn warning(rule_id: &'static str) -> Finding {
        Finding {
            severity: Severity::Warning,
            message: "meh".to_string(),
            ..error(rule_id)
        }
    }

    #[test]
    fn no_findings_exits_success() {
        assert_eq!(Report::new("a.html", &[]).exit_code(), ExitCode::SUCCESS);
    }

    #[test]
    fn only_warnings_exits_two() {
        let report = Report::new("a.html", &[warning("TGT001")]);
        assert_eq!(report.exit_code(), ExitCode::from(2));
    }

    #[test]
    fn any_error_exits_one() {
        let report = Report::new("a.html", &[warning("TGT001"), error("H57")]);
        assert_eq!(report.exit_code(), ExitCode::from(1));
    }

    #[test]
    fn counts_each_severity() {
        let report = Report::new("a.html", &[error("H57"), error("H42"), warning("G18")]);
        assert_eq!((report.errors(), report.warnings()), (2, 1));
    }

    #[rstest]
    #[case("page.html", None, "page.html")]
    #[case("./page.html", None, "page.html")]
    #[case("././dir/page.html", None, "dir/page.html")]
    #[case("/abs/page.html", None, "/abs/page.html")]
    #[case("https://example.com/start", None, "https://example.com/start")]
    #[case(
        "https://example.com/start",
        Some("https://example.com/about"),
        "https://example.com/about"
    )]
    #[case(
        "https://www.example.com/",
        Some("https://example.com/about"),
        "https://example.com/about"
    )]
    fn location_path(#[case] source: &str, #[case] page: Option<&str>, #[case] expected: &str) {
        let page = page.map(|page| Url::parse(page).unwrap());
        assert_eq!(location(source, page.as_ref()).path, expected);
    }

    #[test]
    fn page_query_and_fragment_are_dropped() {
        let report = Report::new(
            "https://example.com",
            &[on_page(error("H57"), "https://example.com/list?page=2#top")],
        );
        assert_eq!(report.issues[0].location.path, "https://example.com/list");
    }

    #[test]
    fn element_is_appended_to_the_message() {
        let finding = Finding {
            element: Some("body > button".to_string()),
            ..error("F68")
        };
        let report = Report::new("a.html", &[finding]);
        assert_eq!(report.issues[0].message, "boom (at body > button)");
    }

    #[test]
    fn same_message_on_different_elements_gets_order_independent_fingerprints() {
        let at = |element: &str| Finding {
            element: Some(element.to_string()),
            ..warning("TGT001")
        };
        let before = Report::new("a.html", &[at("body > a"), at("body > b")]);
        let after = Report::new("a.html", &[at("body > b")]);
        assert_eq!(before.issues[1].fingerprint, after.issues[0].fingerprint);
    }

    #[test]
    fn help_does_not_change_the_message_or_fingerprint() {
        let with_help = Finding {
            help: Some("add an alt attribute".to_string()),
            ..error("F65")
        };
        let plain = Report::new("a.html", &[error("F65")]);
        let helped = Report::new("a.html", &[with_help]);
        assert_eq!(helped.issues[0].message, plain.issues[0].message);
        assert_eq!(helped.issues[0].fingerprint, plain.issues[0].fingerprint);
        assert_eq!(
            helped.issues[0].help.as_deref(),
            Some("add an alt attribute")
        );
    }

    #[test]
    fn issues_link_to_their_rule() {
        let report = Report::new("a.html", &[error("H57"), fatal("FETCH")]);
        assert_eq!(
            report.issues[0].reference.as_deref(),
            Some("https://www.w3.org/WAI/WCAG22/Techniques/html/H57")
        );
        assert_eq!(report.issues[1].reference, None);
    }

    #[test]
    fn full_text_appends_help_and_reference() {
        let finding = Finding {
            help: Some("add lang".to_string()),
            ..error("H57")
        };
        let report = Report::new("a.html", &[finding, fatal("FETCH")]);
        assert_eq!(
            report.issues[0].full_text(),
            "boom\nhelp: add lang\nsee: https://www.w3.org/WAI/WCAG22/Techniques/html/H57"
        );
        assert_eq!(report.issues[1].full_text(), "boom");
    }

    #[test]
    fn location_keeps_the_page() {
        let report = Report::new(
            "https://example.com",
            &[on_page(error("H57"), "https://example.com/about")],
        );
        assert_eq!(report.issues[0].location.page.as_deref(), Some("/about"));
    }

    #[test]
    fn identical_findings_get_distinct_fingerprints() {
        let report = Report::new("a.html", &[error("F65"), error("F65")]);
        assert_ne!(report.issues[0].fingerprint, report.issues[1].fingerprint);
    }

    #[test]
    fn findings_on_different_pages_get_distinct_fingerprints() {
        let report = Report::new(
            "https://example.com",
            &[
                on_page(error("H57"), "https://example.com/a"),
                on_page(error("H57"), "https://example.com/b"),
            ],
        );
        assert_ne!(report.issues[0].fingerprint, report.issues[1].fingerprint);
    }

    #[test]
    fn same_finding_under_another_standard_gets_a_distinct_fingerprint() {
        let report = Report::new(
            "a.html",
            &[error("X1"), fatal("X1").in_standard(Standard::Act)],
        );
        assert_ne!(report.issues[0].fingerprint, report.issues[1].fingerprint);
    }

    #[test]
    fn rule_label_names_the_standard_when_there_is_one() {
        let report = Report::new("a.html", &[error("H57"), fatal("FETCH")]);
        assert_eq!(report.issues[0].rule_label(), "WCAG 2.2 H57");
        assert_eq!(report.issues[1].rule_label(), "FETCH");
    }

    #[test]
    fn fnv1a_matches_reference_vector() {
        assert_eq!(fingerprint(""), "cbf29ce484222325");
        assert_eq!(fingerprint("a"), "af63dc4c8601ec8c");
    }
}
