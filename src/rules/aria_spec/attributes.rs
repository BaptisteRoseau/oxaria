//! Lookup over the WAI-ARIA 1.2 states and properties in
//! [`super::attribute_data`].

use super::attribute_data::ATTRIBUTES;

/// Defined only in the WAI-ARIA 1.3 draft (w3c/aria `main`, commit
/// `7f0665a`), so absent from [`attributes`], but browsers already support
/// some of them: rules can tell these from misspellings.
pub const ARIA_1_3_ATTRIBUTES: &[&str] = &[
    "aria-braillelabel",
    "aria-brailleroledescription",
    "aria-colindextext",
    "aria-description",
    "aria-rowindextext",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    TrueFalse,
    TrueFalseUndefined,
    Tristate,
    IdRef,
    IdRefList,
    Integer,
    Number,
    String,
    Token,
    TokenList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Allowed on every role, except where a role prohibits it.
    Global,
    /// Still global in WAI-ARIA 1.2, but that global use is deprecated:
    /// only the roles listing it support it without deprecation.
    GlobalDeprecated,
    RoleSpecific,
}

#[derive(Debug)]
pub struct Attribute {
    pub name: &'static str,
    /// A state rather than a property.
    pub is_state: bool,
    pub value_type: ValueType,
    /// The spec's values table: the allowed tokens of token types, and the
    /// literal values of true/false, tristate and true/false/undefined.
    pub values: &'static [&'static str],
    pub default: Option<&'static str>,
    pub scope: Scope,
    pub is_deprecated: bool,
}

impl Attribute {
    pub fn is_global(&self) -> bool {
        self.scope != Scope::RoleSpecific
    }
}

pub fn attribute(name: &str) -> Option<&'static Attribute> {
    ATTRIBUTES.iter().find(|attribute| attribute.name == name)
}

pub fn attributes() -> impl Iterator<Item = &'static Attribute> {
    ATTRIBUTES.iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_48_attributes() {
        assert_eq!(attributes().count(), 48);
    }

    #[test]
    fn value_types_and_tokens() {
        let live = attribute("aria-live").unwrap();
        assert_eq!(live.value_type, ValueType::Token);
        assert_eq!(live.values, ["assertive", "off", "polite"]);
        assert_eq!(live.default, Some("off"));
        assert_eq!(
            attribute("aria-checked").unwrap().value_type,
            ValueType::Tristate
        );
        assert_eq!(
            attribute("aria-relevant").unwrap().values,
            ["additions", "all", "removals", "text"]
        );
    }

    #[test]
    fn global_attributes_include_the_deprecated_globals() {
        let global: Vec<_> = attributes()
            .filter(|a| a.is_global())
            .map(|a| a.name)
            .collect();
        assert_eq!(global.len(), 21);
        assert!(global.contains(&"aria-label"));
        assert!(global.contains(&"aria-disabled"));
        assert!(!global.contains(&"aria-checked"));
        assert_eq!(
            attribute("aria-haspopup").unwrap().scope,
            Scope::GlobalDeprecated
        );
    }

    #[test]
    fn only_drag_and_drop_attributes_are_deprecated() {
        let deprecated: Vec<_> = attributes()
            .filter(|a| a.is_deprecated)
            .map(|a| a.name)
            .collect();
        assert_eq!(deprecated, ["aria-dropeffect", "aria-grabbed"]);
    }

    #[test]
    fn aria_1_3_attributes_are_not_aria_1_2_ones() {
        assert!(
            ARIA_1_3_ATTRIBUTES
                .iter()
                .all(|name| attribute(name).is_none())
        );
    }
}
