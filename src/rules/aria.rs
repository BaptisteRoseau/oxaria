//! WCAG 4.1.1 / 4.1.2 checks for `id` uniqueness and ARIA references.

use std::collections::BTreeMap;

use crate::page::RenderedPage;

use super::{CheckOptions, Finding};

const REFERENCE_ATTRIBUTES: &[&str] = &["aria-labelledby", "aria-describedby"];

/// F77: duplicate `id` values break `for`, `aria-labelledby`/`aria-describedby`, and fragment
/// navigation, since only one element can be resolved for a given id.
pub fn check_duplicate_ids(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    id_counts(page)
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(id, count)| Finding::error("F77", duplicate_id_message(id, count)))
        .collect()
}

fn id_counts(page: &RenderedPage) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for id in page.all().filter_map(|el| el.attr("id")) {
        *counts.entry(id).or_insert(0) += 1;
    }
    counts
}

fn duplicate_id_message(id: &str, count: usize) -> String {
    format!("id \"{id}\" is used {count} times, but ids must be unique")
}

/// ARIA16: `aria-labelledby`/`aria-describedby` must reference an id that actually exists,
/// otherwise the accessible name/description computation silently fails.
pub fn check_dangling_aria_reference(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    REFERENCE_ATTRIBUTES
        .iter()
        .flat_map(|attribute| dangling_references(page, attribute))
        .collect()
}

fn dangling_references(page: &RenderedPage, attribute: &str) -> Vec<Finding> {
    page.all()
        .filter(|el| el.has_attr(attribute))
        .flat_map(|element| {
            element
                .attr(attribute)
                .unwrap_or_default()
                .split_whitespace()
                .filter(|id| page.element_by_id(id).is_none())
                .map(move |id| {
                    Finding::error("ARIA16", dangling_reference_message(attribute, id)).at(element)
                })
        })
        .collect()
}

fn dangling_reference_message(attribute: &str, id: &str) -> String {
    format!("{attribute} references id \"{id}\", which does not exist in the document")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn duplicate_ids_are_flagged() {
        let p = page_from_html(r#"<input id="search"><input id="search">"#);
        let findings = check_duplicate_ids(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F77");
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
