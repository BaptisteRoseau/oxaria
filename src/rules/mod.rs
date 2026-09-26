//! Rule registry and orchestration. Every rule is a plain function taking the
//! rendered page and the CLI-configured [`CheckOptions`], returning the
//! [`Finding`]s it detected -- there is no distinction between rules that
//! only need DOM/attribute data and rules that need computed style or
//! layout, since every rule reads from the same [`RenderedPage`] snapshot.
//! [`run_all`] spawns one `tokio::task` per rule so they run in parallel.

mod aria;
mod contrast;
mod focus;
mod forms;
mod headings;
mod images;
mod language;
mod links;
mod multimedia;
mod navigation;
mod tables;
mod target_size;

use std::sync::Arc;

use tracing::error;
use url::Url;

use crate::cli::CliConfig;
use crate::page::{ElementRef, RenderedPage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub message: String,
    /// Selector-like path of the element the finding is about, for rules
    /// that point at one element (not page-level or stylesheet rules).
    pub element: Option<String>,
    /// URL of the page this finding belongs to, as it was actually served
    /// (after redirects); only set during a full-site scan, where a single
    /// report covers many pages.
    pub page: Option<Url>,
}

impl Finding {
    pub(crate) fn error(rule_id: &'static str, message: String) -> Self {
        Finding {
            rule_id,
            severity: Severity::Error,
            message,
            element: None,
            page: None,
        }
    }

    fn warning(rule_id: &'static str, message: String) -> Self {
        Finding {
            severity: Severity::Warning,
            ..Finding::error(rule_id, message)
        }
    }

    fn at(self, element: ElementRef) -> Self {
        Finding {
            element: Some(element.selector()),
            ..self
        }
    }

    /// Query and fragment are dropped: the crawl treats URLs differing only
    /// by them as the same page.
    pub fn on_page(self, page: &Url) -> Self {
        let mut page = page.clone();
        page.set_query(None);
        page.set_fragment(None);
        Finding {
            page: Some(page),
            ..self
        }
    }

    #[cfg(test)]
    pub fn page_path(&self) -> Option<&str> {
        self.page.as_ref().map(Url::path)
    }
}

pub const DEFAULT_CONTRAST_THRESHOLD: f64 = 4.5;
pub const DEFAULT_LARGE_TEXT_CONTRAST_THRESHOLD: f64 = 3.0;
pub const DEFAULT_TARGET_SIZE_THRESHOLD: f64 = 24.0;

pub struct CheckOptions {
    pub contrast_threshold: f64,
    pub large_text_contrast_threshold: f64,
    pub target_size_threshold: f64,
}

impl Default for CheckOptions {
    fn default() -> Self {
        CheckOptions {
            contrast_threshold: DEFAULT_CONTRAST_THRESHOLD,
            large_text_contrast_threshold: DEFAULT_LARGE_TEXT_CONTRAST_THRESHOLD,
            target_size_threshold: DEFAULT_TARGET_SIZE_THRESHOLD,
        }
    }
}

impl From<&CliConfig> for CheckOptions {
    fn from(config: &CliConfig) -> Self {
        CheckOptions {
            contrast_threshold: config.contrast_threshold,
            large_text_contrast_threshold: config.large_text_contrast_threshold,
            target_size_threshold: config.target_size_threshold,
        }
    }
}

type RuleCheck = fn(&RenderedPage, &CheckOptions) -> Vec<Finding>;

fn all_rule_checks() -> Vec<RuleCheck> {
    vec![
        images::check_missing_alt,
        images::check_non_alternative_alt,
        forms::check_missing_label,
        forms::check_required_not_indicated,
        forms::check_unnamed_control,
        headings::check_missing_h1,
        headings::check_skipped_heading_level,
        language::check_missing_lang,
        links::check_non_descriptive_link_text,
        links::check_ambiguous_duplicate_link_text,
        contrast::check_text_contrast,
        tables::check_table_missing_headers,
        tables::check_header_missing_scope,
        aria::check_duplicate_ids,
        aria::check_dangling_aria_reference,
        multimedia::check_video_missing_captions,
        multimedia::check_autoplay_without_controls,
        focus::check_outline_removed_without_alternative,
        navigation::check_missing_skip_link,
        target_size::check_target_size,
    ]
}

/// Runs every rule in its own `tokio::task`. [`RenderedPage`] is plain owned
/// data (no borrowed/FFI handles), so it is genuinely `Send + Sync` and can
/// be shared across tasks through the same `Arc` directly.
pub async fn run_all(page: Arc<RenderedPage>, options: Arc<CheckOptions>) -> Vec<Finding> {
    let tasks: Vec<_> = all_rule_checks()
        .into_iter()
        .map(|check| spawn_rule(check, Arc::clone(&page), Arc::clone(&options)))
        .collect();

    let mut findings = Vec::new();
    for task in tasks {
        match task.await {
            Ok(mut rule_findings) => findings.append(&mut rule_findings),
            Err(join_error) => error!("rule task panicked: {join_error}"),
        }
    }
    findings
}

fn spawn_rule(
    check: RuleCheck,
    page: Arc<RenderedPage>,
    options: Arc<CheckOptions>,
) -> tokio::task::JoinHandle<Vec<Finding>> {
    tokio::task::spawn_blocking(move || check(&page, &options))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn options() -> Arc<CheckOptions> {
        Arc::new(CheckOptions::default())
    }

    #[tokio::test]
    async fn run_all_returns_findings_from_every_rule_kind() {
        let empty = Arc::new(page_from_html("<p>Nothing here</p>"));
        let findings = run_all(empty, options()).await;
        assert!(findings.iter().any(|f| f.rule_id == "H57"));
        assert!(findings.iter().any(|f| f.rule_id == "H42"));
    }

    #[tokio::test]
    async fn run_all_returns_no_findings_for_a_clean_document() {
        let clean = Arc::new(page_from_html(
            r##"<html lang="en"><a href="#main">Skip to main content</a>
               <h1 id="main">Title</h1></html>"##,
        ));
        let findings = run_all(clean, options()).await;
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    #[tokio::test]
    async fn element_findings_point_at_their_element_page_findings_do_not() {
        let page = Arc::new(page_from_html(
            r#"<html lang="en"><body><h1>T</h1><div><button></button></div></body></html>"#,
        ));
        let findings = run_all(page, options()).await;
        let element_of = |rule| {
            findings
                .iter()
                .find(|f| f.rule_id == rule)
                .map(|f| f.element.as_deref())
        };
        assert_eq!(element_of("F68"), Some(Some("body > div > button")));
        assert_eq!(element_of("G1"), Some(None));
    }
}
