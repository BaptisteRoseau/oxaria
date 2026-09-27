use std::sync::Arc;

use tokio::task::JoinHandle;
use tracing::error;

use super::{CheckOptions, Finding, Standard, act, aria12, html_aria, wcag22};
use crate::page::RenderedPage;

pub type RuleCheck = fn(&RenderedPage, &CheckOptions) -> Vec<Finding>;

fn rule_checks(standard: Standard) -> Vec<RuleCheck> {
    match standard {
        Standard::Wcag22 => wcag22::rule_checks(),
        Standard::Aria12 => aria12::rule_checks(),
        Standard::HtmlAria => html_aria::rule_checks(),
        Standard::Act => act::rule_checks(),
    }
}

fn rule_ids(standard: Standard) -> &'static [&'static str] {
    match standard {
        Standard::Wcag22 => wcag22::RULE_IDS,
        Standard::Aria12 => aria12::RULE_IDS,
        Standard::HtmlAria => html_aria::RULE_IDS,
        Standard::Act => act::RULE_IDS,
    }
}

fn rules_file(standard: Standard) -> &'static str {
    match standard {
        Standard::Wcag22 => include_str!("../../standards/wcag2.2-rules.md"),
        Standard::Aria12 => include_str!("../../standards/wai-aria-1.2-rules.md"),
        Standard::HtmlAria => include_str!("../../standards/html-aria-rules.md"),
        Standard::Act => include_str!("../../standards/act-rules.md"),
    }
}

/// `(rule ID, title)` of every rule of `standard`, each title taken from the
/// `### <ID> - <title>` heading of its rule file under `standards/`.
pub fn rule_titles(standard: Standard) -> Vec<(&'static str, &'static str)> {
    rule_ids(standard)
        .iter()
        .map(|&id| (id, title(rules_file(standard), id).unwrap_or_default()))
        .collect()
}

fn title(rules_file: &'static str, rule_id: &str) -> Option<&'static str> {
    rules_file
        .lines()
        .filter_map(|line| line.strip_prefix("### ")?.split_once(" - "))
        .find(|(ids, _)| ids.split(" / ").any(|id| id == rule_id))
        .map(|(_, title)| title)
}

/// Runs every rule of the selected standards in its own blocking task, sharing one
/// `Arc<RenderedPage>`.
pub async fn run_all(page: Arc<RenderedPage>, options: Arc<CheckOptions>) -> Vec<Finding> {
    let tasks: Vec<_> = options
        .standards
        .iter()
        .flat_map(|&standard| {
            rule_checks(standard)
                .into_iter()
                .map(move |check| (standard, check))
        })
        .map(|(standard, check)| {
            let task = spawn_rule(standard, check, Arc::clone(&page), Arc::clone(&options));
            (standard, task)
        })
        .collect();

    let mut findings = Vec::new();
    for (standard, task) in tasks {
        match task.await {
            Ok(mut rule_findings) => findings.append(&mut rule_findings),
            Err(join_error) => error!("{standard} rule task panicked: {join_error}"),
        }
    }
    findings
}

fn spawn_rule(
    standard: Standard,
    check: RuleCheck,
    page: Arc<RenderedPage>,
    options: Arc<CheckOptions>,
) -> JoinHandle<Vec<Finding>> {
    tokio::task::spawn_blocking(move || {
        check(&page, &options)
            .into_iter()
            .map(|finding| finding.in_standard(standard))
            .collect()
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeSet;

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

    /// Fails unless the pages make exactly the rule IDs of `standard` fire,
    /// each finding with a help line.
    pub(crate) async fn assert_every_rule_has_help(standard: Standard, pages: &[&str]) {
        let mut findings = Vec::new();
        for html in pages {
            findings.extend(run_all(Arc::new(page_from_html(html)), options()).await);
        }
        findings.retain(|f| f.standard == Some(standard));
        let rule_ids: BTreeSet<_> = findings.iter().map(|f| f.rule_id).collect();
        assert_eq!(
            rule_ids,
            super::rule_ids(standard).iter().copied().collect(),
            "not every {standard} rule fired"
        );
        let without_help: Vec<_> = findings.iter().filter(|f| f.help.is_none()).collect();
        assert!(without_help.is_empty(), "{without_help:?}");
    }

    #[test]
    fn every_rule_has_a_title() {
        for standard in Standard::ALL {
            let untitled: Vec<_> = rule_titles(standard)
                .into_iter()
                .filter(|(_, title)| title.is_empty())
                .collect();
            assert!(untitled.is_empty(), "{standard}: {untitled:?}");
        }
    }

    #[test]
    fn title_is_found_under_a_shared_heading() {
        assert_eq!(
            title(rules_file(Standard::Wcag22), "F41"),
            Some("Don't redirect or reload the page with a timed meta refresh")
        );
    }

    #[tokio::test]
    async fn run_all_runs_only_the_selected_standards() {
        let page = Arc::new(page_from_html("<p>Nothing here</p>"));
        let options = Arc::new(CheckOptions {
            standards: BTreeSet::from([Standard::Act]),
            ..CheckOptions::default()
        });
        let findings = run_all(page, options).await;
        assert!(!findings.is_empty());
        assert!(
            findings.iter().all(|f| f.standard == Some(Standard::Act)),
            "{findings:?}"
        );
    }

    #[tokio::test]
    async fn every_finding_is_stamped_with_its_standard() {
        let page = Arc::new(page_from_html("<p>Nothing here</p>"));
        let findings = run_all(page, options()).await;
        assert!(!findings.is_empty());
        assert!(
            findings.iter().all(|f| f.standard.is_some()),
            "{findings:?}"
        );
        let h57 = findings.iter().find(|f| f.rule_id == "H57").unwrap();
        assert_eq!(h57.standard, Some(Standard::Wcag22));
    }

    #[tokio::test]
    async fn run_all_returns_no_findings_for_a_clean_document() {
        let clean = Arc::new(page_from_html(
            r##"<html lang="en"><title>Clean page</title><a href="#main">Skip to main content</a>
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
