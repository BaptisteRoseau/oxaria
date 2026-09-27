const SPEC_URL: &str = "https://www.w3.org/TR/html-aria/";

/// Rule ID -> anchor of the spec section the rule cites in
/// `standards/html-aria-rules.md`. HTMLARIA017's section has no id in the
/// spec source: ReSpec derives it from the heading.
const ANCHORS: &[(&str, &str)] = &[
    ("HTMLARIA002", "rules-wd"),
    ("HTMLARIA003", "docconformance"),
    ("HTMLARIA004", "conformance"),
    ("HTMLARIA007", "docconformance"),
    ("HTMLARIA009", "att-hidden"),
    ("HTMLARIA010", "att-hidden"),
    ("HTMLARIA011", "docconformance"),
    ("HTMLARIA013", "docconformance-attr"),
    ("HTMLARIA014", "docconformance-attr"),
    ("HTMLARIA015", "docconformance-deprecated"),
    ("HTMLARIA016", "case-sensitivity"),
    ("HTMLARIA017", "allowed-descendants-of-aria-roles"),
];

/// Rules without an entry in [`ANCHORS`] link to the spec itself.
pub fn reference_url(rule_id: &str) -> String {
    match ANCHORS.iter().find(|(id, _)| *id == rule_id) {
        Some((_, anchor)) => format!("{SPEC_URL}#{anchor}"),
        None => SPEC_URL.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_link_to_the_section_they_cite() {
        assert_eq!(
            reference_url("HTMLARIA013"),
            "https://www.w3.org/TR/html-aria/#docconformance-attr"
        );
    }

    #[test]
    fn rules_without_an_anchor_link_to_the_spec() {
        assert_eq!(reference_url("UNKNOWN"), SPEC_URL);
    }
}
