const SPEC_URL: &str = "https://www.w3.org/TR/wai-aria-1.2/";

/// Rule ID -> anchor of the spec section the rule cites in
/// `standards/wai-aria-1.2-rules.md`.
const ANCHORS: &[(&str, &str)] = &[];

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
    fn rules_without_an_anchor_link_to_the_spec() {
        assert_eq!(reference_url("UNKNOWN"), SPEC_URL);
    }
}
