//! Shared helpers for rule unit tests: each runs a check against a page built
//! from plain HTML by [`page_from_html`].

use super::{CheckOptions, Finding, RuleCheck};
use crate::page::RenderedPage;
use crate::page::testutil::page_from_html;

pub fn run(check: RuleCheck, html: &str) -> Vec<Finding> {
    check(&page_from_html(html))
}

/// Runs a threshold rule with the default CLI options.
pub fn run_with_defaults(
    check: fn(&RenderedPage, &CheckOptions) -> Vec<Finding>,
    html: &str,
) -> Vec<Finding> {
    check(&page_from_html(html), &CheckOptions::default())
}

pub fn count(check: RuleCheck, html: &str) -> usize {
    run(check, html).len()
}

/// The help of the first finding.
pub fn help(check: RuleCheck, html: &str) -> String {
    run(check, html)[0].help.clone().unwrap()
}

/// Asserts `findings` is exactly one finding of `rule_id`, and returns it.
pub fn assert_one(findings: Vec<Finding>, rule_id: &str) -> Finding {
    let [finding] = <[Finding; 1]>::try_from(findings)
        .unwrap_or_else(|findings| panic!("expected one {rule_id} finding, got {findings:?}"));
    assert_eq!(finding.rule_id, rule_id);
    finding
}

/// Asserts `check` reports exactly one finding of `rule_id` on `html`, and returns it.
pub fn assert_fires(check: RuleCheck, html: &str, rule_id: &str) -> Finding {
    assert_one(run(check, html), rule_id)
}

/// The rule ID of every finding `check` reports on `html`, in order.
pub fn rule_ids(check: RuleCheck, html: &str) -> Vec<&'static str> {
    run(check, html)
        .iter()
        .map(|finding| finding.rule_id)
        .collect()
}
