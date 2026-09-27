const SPEC_URL: &str = "https://www.w3.org/TR/wai-aria-1.2/";

/// Rule ID -> anchor of the spec section the rule cites in
/// `standards/wai-aria-1.2-rules.md`.
const ANCHORS: &[(&str, &str)] = &[
    ("ARIA-ROLE002", "isAbstract"),
    ("ARIA-ROLE003", "generic"),
    ("ARIA-ATTR001", "host_general_attrs"),
    ("ARIA-ATTR003", "supportedState"),
    ("ARIA-ATTR004", "prohibitedattributes"),
    ("ARIA-ATTR005", "row"),
    ("ARIA-ATTR007", "aria-keyshortcuts"),
    ("ARIA-ATTR008", "aria-roledescription"),
    ("ARIA-ATTR009", "aria-current"),
    ("ARIA-VAL001", "propcharacteristic_value"),
    ("ARIA-VAL002", "aria-posinset"),
    ("ARIA-VAL003", "aria-valuenow"),
    ("ARIA-VAL005", "aria-posinset"),
    (
        "ARIA-IDREF001",
        "mapping_additional_relations_error_processing",
    ),
    ("ARIA-IDREF002", "aria-owns"),
    ("ARIA-IDREF003", "aria-activedescendant"),
    ("ARIA-STRUCT001", "scope"),
    ("ARIA-STRUCT002", "mustContain"),
    ("ARIA-STRUCT003", "group"),
    ("ARIA-STRUCT004", "caption"),
    ("ARIA-PRES001", "conflict_resolution_presentation_none"),
    ("ARIA-PRES002", "presentation"),
    ("ARIA-PRES003", "childrenArePresentational"),
    ("ARIA-WIDGET001", "combobox"),
    ("ARIA-WIDGET002", "aria-haspopup"),
    ("ARIA-WIDGET003", "aria-autocomplete"),
    ("ARIA-WIDGET006", "aria-haspopup"),
    ("ARIA-WIDGET007", "feed"),
    ("ARIA-FOCUS001", "managingfocus_authors"),
    ("ARIA-FOCUS006", "dialog"),
    ("ARIA-LMK001", "main"),
    ("ARIA-USAGE002", "table"),
    ("ARIA-USAGE003", "aria-labelledby"),
    ("ARIA-DEPR001", "directory"),
    ("ARIA-DEPR002", "aria-grabbed"),
    ("ARIA-DEPR003", "aria-disabled"),
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
    fn rules_without_an_anchor_link_to_the_spec() {
        assert_eq!(reference_url("UNKNOWN"), SPEC_URL);
    }

    #[test]
    fn rules_link_to_the_section_they_cite() {
        assert_eq!(
            reference_url("ARIA-PRES003"),
            "https://www.w3.org/TR/wai-aria-1.2/#childrenArePresentational"
        );
    }

    #[test]
    fn every_rule_has_an_anchor() {
        assert_eq!(ANCHORS.len(), super::super::rule_checks().len());
    }
}
