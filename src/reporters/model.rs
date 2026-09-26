//! The format-independent view of a run that every [`Reporter`] renders:
//! each [`Finding`] enriched with where it was found and a stable
//! fingerprint, plus the exit code derived from them.
//!
//! [`Reporter`]: super::Reporter

use std::collections::HashMap;
use std::process::ExitCode;

use url::Url;

use crate::page;
use crate::rules::{Finding, Severity};

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
    pub severity: Severity,
    pub message: String,
    pub location: Location,
    pub fingerprint: String,
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
                let location = location(source, finding.page.as_deref());
                let key = fingerprint_key(finding, &location);
                let occurrence = occurrences.entry(key.clone()).or_insert(0usize);
                *occurrence += 1;
                Issue {
                    rule_id: finding.rule_id,
                    severity: finding.severity,
                    message: finding.message.clone(),
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

fn location(source: &str, page: Option<&str>) -> Location {
    let path = match (page::is_url(source), page) {
        (true, Some(page)) => page_url(source, page),
        (true, None) => source.to_string(),
        (false, _) => strip_current_dir(source).to_string(),
    };
    Location {
        path,
        page: page.map(str::to_string),
    }
}

/// The crawl only records the page's path, so it is grafted onto the source
/// URL's origin (a start URL redirecting to another host keeps the origin
/// it was given on the command line).
fn page_url(source: &str, page: &str) -> String {
    match Url::parse(source) {
        Ok(mut url) => {
            url.set_path(page);
            url.set_query(None);
            url.set_fragment(None);
            url.to_string()
        }
        Err(_) => format!("{}{page}", source.trim_end_matches('/')),
    }
}

fn strip_current_dir(mut path: &str) -> &str {
    while let Some(rest) = path.strip_prefix("./") {
        path = rest;
    }
    path
}

fn fingerprint_key(finding: &Finding, location: &Location) -> String {
    [
        finding.rule_id,
        &location.path,
        location.page.as_deref().unwrap_or(""),
        &finding.message,
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
        Finding::error(rule_id, "boom".to_string())
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
        "https://example.com/start?q=1#top",
        Some("/about"),
        "https://example.com/about"
    )]
    fn location_path(#[case] source: &str, #[case] page: Option<&str>, #[case] expected: &str) {
        assert_eq!(location(source, page).path, expected);
    }

    #[test]
    fn location_keeps_the_page() {
        let report = Report::new(
            "https://example.com",
            &[error("H57").on_page("/about".to_string())],
        );
        assert_eq!(report.issues[0].location.page.as_deref(), Some("/about"));
    }

    #[test]
    fn fingerprints_are_stable_across_runs() {
        let first = Report::new("a.html", &[error("H57")]);
        let second = Report::new("a.html", &[error("H57")]);
        assert_eq!(first.issues[0].fingerprint, second.issues[0].fingerprint);
        assert_eq!(first.issues[0].fingerprint.len(), 16);
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
                error("H57").on_page("/a".to_string()),
                error("H57").on_page("/b".to_string()),
            ],
        );
        assert_ne!(report.issues[0].fingerprint, report.issues[1].fingerprint);
    }

    #[test]
    fn fnv1a_matches_reference_vector() {
        assert_eq!(fingerprint(""), "cbf29ce484222325");
        assert_eq!(fingerprint("a"), "af63dc4c8601ec8c");
    }
}
