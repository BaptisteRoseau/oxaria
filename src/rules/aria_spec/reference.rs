/// `spec_url#anchor` for a rule with an entry in `anchors`, otherwise the spec
/// itself.
pub fn anchored_url(spec_url: &str, anchors: &[(&str, &str)], rule_id: &str) -> Option<String> {
    Some(match anchors.iter().find(|(id, _)| *id == rule_id) {
        Some((_, anchor)) => format!("{spec_url}#{anchor}"),
        None => spec_url.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC_URL: &str = "https://example.org/spec/";
    const ANCHORS: &[(&str, &str)] = &[("RULE001", "section")];

    #[test]
    fn rules_link_to_the_section_they_cite() {
        assert_eq!(
            anchored_url(SPEC_URL, ANCHORS, "RULE001").as_deref(),
            Some("https://example.org/spec/#section")
        );
    }

    #[test]
    fn rules_without_an_anchor_link_to_the_spec() {
        assert_eq!(
            anchored_url(SPEC_URL, ANCHORS, "UNKNOWN").as_deref(),
            Some(SPEC_URL)
        );
    }
}
