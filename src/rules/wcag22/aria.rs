//! WCAG 4.1.1 / 4.1.2 checks for `id` uniqueness and ARIA references.

use crate::page::RenderedPage;

use crate::rules::Finding;

/// Each reference attribute with the technique that uses it.
const REFERENCE_ATTRIBUTES: &[(&str, &str)] =
    &[("aria-labelledby", "ARIA16"), ("aria-describedby", "ARIA1")];

/// IDS001: duplicate `id` values break `for`, `aria-labelledby`/`aria-describedby`, and fragment
/// navigation, since only one element can be resolved for a given id.
pub fn check_duplicate_ids(page: &RenderedPage) -> Vec<Finding> {
    let mut duplicates: Vec<(&str, usize)> =
        page.id_counts().filter(|(_, count)| *count > 1).collect();
    duplicates.sort_unstable();
    duplicates
        .into_iter()
        .map(|(id, count)| {
            Finding::error("IDS001", duplicate_id_message(id, count)).help(duplicate_id_help(id))
        })
        .collect()
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
pub fn check_dangling_aria_reference(page: &RenderedPage) -> Vec<Finding> {
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
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::{help, rule_ids, run};

    #[test]
    fn dangling_reference_help_names_the_missing_id() {
        let html = r#"<button aria-describedby="hint">Go</button>"#;
        assert!(help(check_dangling_aria_reference, html).starts_with(r#"add id="hint" to"#));
    }

    #[test]
    fn duplicate_ids_are_reported_in_a_stable_order() {
        let html = r#"<i id="c"></i><i id="c"></i><i id="a"></i><i id="a"></i><i id="b"></i><i id="b"></i>"#;
        let messages: Vec<_> = run(check_duplicate_ids, html)
            .into_iter()
            .map(|f| f.message)
            .collect();
        assert!(messages[0].contains("\"a\""), "{messages:?}");
        assert!(messages[1].contains("\"b\""), "{messages:?}");
        assert!(messages[2].contains("\"c\""), "{messages:?}");
    }

    #[rstest]
    #[case(r#"<input id="search"><input id="search">"#, &["IDS001"])]
    #[case(r#"<input id="a"><input id="b">"#, &[])]
    fn duplicate_ids(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_duplicate_ids, html), expected, "{html}");
    }

    #[rstest]
    #[case(r#"<input aria-labelledby="missing">"#, &["ARIA16"])]
    #[case(r#"<input aria-describedby="missing">"#, &["ARIA1"])]
    #[case(r#"<span id="lbl">Name</span><input aria-labelledby="lbl">"#, &[])]
    fn dangling_aria_reference(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(
            rule_ids(check_dangling_aria_reference, html),
            expected,
            "{html}"
        );
    }
}
