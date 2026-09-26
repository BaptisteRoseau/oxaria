//! WCAG 4.1.1 / 4.1.2 checks for `id` uniqueness and ARIA references.

use std::collections::BTreeMap;

use crate::page::RenderedPage;

use crate::rules::{CheckOptions, Finding};

/// Each reference attribute with the technique that uses it.
const REFERENCE_ATTRIBUTES: &[(&str, &str)] =
    &[("aria-labelledby", "ARIA16"), ("aria-describedby", "ARIA1")];

/// IDS001: duplicate `id` values break `for`, `aria-labelledby`/`aria-describedby`, and fragment
/// navigation, since only one element can be resolved for a given id.
pub fn check_duplicate_ids(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    id_counts(page)
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(id, count)| {
            Finding::error("IDS001", duplicate_id_message(id, count)).help(duplicate_id_help(id))
        })
        .collect()
}

fn id_counts(page: &RenderedPage) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for id in page.all().filter_map(|el| el.attr("id")) {
        *counts.entry(id).or_insert(0) += 1;
    }
    counts
}

fn duplicate_id_help(id: &str) -> String {
    format!(
        "give each element its own id (e.g. \"{id}-1\", \"{id}-2\"), and update the for=, \
         aria-labelledby/aria-describedby and \"#{id}\" references to match"
    )
}

fn duplicate_id_message(id: &str, count: usize) -> String {
    format!("id \"{id}\" is used {count} times, but ids must be unique")
}

/// ARIA16/ARIA1: `aria-labelledby`/`aria-describedby` must reference an id that actually exists,
/// otherwise the accessible name/description computation silently fails.
pub fn check_dangling_aria_reference(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    REFERENCE_ATTRIBUTES
        .iter()
        .flat_map(|(attribute, rule_id)| dangling_references(page, attribute, rule_id))
        .collect()
}

fn dangling_references(
    page: &RenderedPage,
    attribute: &str,
    rule_id: &'static str,
) -> Vec<Finding> {
    page.all()
        .filter(|el| el.has_attr(attribute))
        .flat_map(|element| {
            element
                .attr(attribute)
                .unwrap_or_default()
                .split_whitespace()
                .filter(|id| page.element_by_id(id).is_none())
                .map(move |id| {
                    Finding::error(rule_id, dangling_reference_message(attribute, id))
                        .at(element)
                        .help(dangling_reference_help(attribute, id))
                })
        })
        .collect()
}

fn dangling_reference_help(attribute: &str, id: &str) -> String {
    format!(
        "add id=\"{id}\" to the element holding the text, or fix or remove the reference in \
         {attribute}"
    )
}

fn dangling_reference_message(attribute: &str, id: &str) -> String {
    format!("{attribute} references id \"{id}\", which does not exist in the document")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn dangling_reference_help_names_the_missing_id() {
        let p = page_from_html(r#"<button aria-describedby="hint">Go</button>"#);
        let findings = check_dangling_aria_reference(&p, &CheckOptions::default());
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with(r#"add id="hint" to"#)
        );
    }

    #[test]
    fn duplicate_ids_are_flagged() {
        let p = page_from_html(r#"<input id="search"><input id="search">"#);
        let findings = check_duplicate_ids(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "IDS001");
    }

    #[test]
    fn duplicate_ids_are_reported_in_a_stable_order() {
        let p = page_from_html(
            r#"<i id="c"></i><i id="c"></i><i id="a"></i><i id="a"></i><i id="b"></i><i id="b"></i>"#,
        );
        let messages: Vec<_> = check_duplicate_ids(&p, &CheckOptions::default())
            .into_iter()
            .map(|f| f.message)
            .collect();
        assert!(messages[0].contains("\"a\""), "{messages:?}");
        assert!(messages[1].contains("\"b\""), "{messages:?}");
        assert!(messages[2].contains("\"c\""), "{messages:?}");
    }

    #[test]
    fn unique_ids_are_not_flagged() {
        let p = page_from_html(r#"<input id="a"><input id="b">"#);
        assert!(check_duplicate_ids(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn dangling_aria_labelledby_is_flagged() {
        let p = page_from_html(r#"<input aria-labelledby="missing">"#);
        let findings = check_dangling_aria_reference(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "ARIA16");
    }

    #[test]
    fn dangling_aria_describedby_is_reported_as_aria1() {
        let p = page_from_html(r#"<input aria-describedby="missing">"#);
        let findings = check_dangling_aria_reference(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "ARIA1");
    }

    #[test]
    fn resolved_aria_labelledby_is_not_flagged() {
        let p = page_from_html(r#"<span id="lbl">Name</span><input aria-labelledby="lbl">"#);
        assert!(check_dangling_aria_reference(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn dangling_aria_describedby_is_flagged() {
        let p = page_from_html(r#"<input aria-describedby="missing">"#);
        assert_eq!(
            check_dangling_aria_reference(&p, &CheckOptions::default()).len(),
            1
        );
    }
}
