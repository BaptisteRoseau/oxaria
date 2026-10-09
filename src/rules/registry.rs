use std::sync::Arc;

use tokio::task::JoinHandle;
use tracing::error;

use super::{CheckOptions, Finding, Standard};
use crate::page::RenderedPage;

pub type RuleCheck = fn(&RenderedPage) -> Vec<Finding>;

/// A rule's check; only the threshold-based rules read the CLI options.
#[derive(Clone, Copy)]
pub enum Check {
    Page(RuleCheck),
    WithOptions(fn(&RenderedPage, &CheckOptions) -> Vec<Finding>),
}

/// One rule: its check and every rule ID its findings can carry.
pub struct Rule {
    pub ids: &'static [&'static str],
    pub check: Check,
}

impl Rule {
    pub const fn new(ids: &'static [&'static str], check: RuleCheck) -> Self {
        Self {
            ids,
            check: Check::Page(check),
        }
    }

    pub const fn with_options(
        ids: &'static [&'static str],
        check: fn(&RenderedPage, &CheckOptions) -> Vec<Finding>,
    ) -> Self {
        Self {
            ids,
            check: Check::WithOptions(check),
        }
    }
}

impl Check {
    pub fn run(self, page: &RenderedPage, options: &CheckOptions) -> Vec<Finding> {
        match self {
            Check::Page(check) => check(page),
            Check::WithOptions(check) => check(page, options),
        }
    }
}

fn rule_ids(standard: Standard) -> impl Iterator<Item = &'static str> {
    standard
        .def()
        .rules
        .iter()
        .flat_map(|rule| rule.ids.iter().copied())
}

/// `(rule ID, title)` of every rule of `standard`, each title taken from the
/// `### <ID> - <title>` heading of its rule file under `standards/`.
pub fn rule_titles(standard: Standard) -> Vec<(&'static str, &'static str)> {
    rule_ids(standard)
        .map(|id| (id, title(standard.def().rules_file, id).unwrap_or_default()))
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
            standard
                .def()
                .rules
                .iter()
                .map(move |rule| (standard, rule.check))
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
    check: Check,
    page: Arc<RenderedPage>,
    options: Arc<CheckOptions>,
) -> JoinHandle<Vec<Finding>> {
    tokio::task::spawn_blocking(move || {
        check
            .run(&page, &options)
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
    /// each finding with a help line and an ID declared by the rule that produced it.
    pub(crate) async fn assert_every_rule_has_help(standard: Standard, pages: &[&str]) {
        let options = CheckOptions::default();
        let mut findings = Vec::new();
        for html in pages {
            let page = page_from_html(html);
            for rule in standard.def().rules {
                let rule_findings = rule.check.run(&page, &options);
                let undeclared: Vec<_> = rule_findings
                    .iter()
                    .filter(|f| !rule.ids.contains(&f.rule_id))
                    .collect();
                assert!(undeclared.is_empty(), "{:?}: {undeclared:?}", rule.ids);
                findings.extend(rule_findings);
            }
        }
        let rule_ids: BTreeSet<_> = findings.iter().map(|f| f.rule_id).collect();
        assert_eq!(
            rule_ids,
            super::rule_ids(standard).collect(),
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
            title(Standard::Wcag22.def().rules_file, "F41"),
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
