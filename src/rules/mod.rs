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

use crate::cli::CliConfig;
use crate::page::RenderedPage;

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
    /// URL path of the page this finding belongs to; only set during a
    /// full-site scan, where a single report covers many pages.
    pub page: Option<String>,
}

impl Finding {
    pub(crate) fn error(rule_id: &'static str, message: String) -> Self {
        Finding {
            rule_id,
            severity: Severity::Error,
            message,
            page: None,
        }
    }

    fn warning(rule_id: &'static str, message: String) -> Self {
        Finding {
            rule_id,
            severity: Severity::Warning,
            message,
            page: None,
        }
    }

    pub fn on_page(self, page: String) -> Self {
        Finding {
            page: Some(page),
            ..self
        }
    }
}

pub struct CheckOptions {
    pub contrast_threshold: f64,
    pub large_text_contrast_threshold: f64,
    pub target_size_threshold: f64,
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
    tokio::spawn(async move { check(&page, &options) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn options() -> Arc<CheckOptions> {
        Arc::new(CheckOptions {
            contrast_threshold: 4.5,
            large_text_contrast_threshold: 3.0,
            target_size_threshold: 24.0,
        })
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
}
