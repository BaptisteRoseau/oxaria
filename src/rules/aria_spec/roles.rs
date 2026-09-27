//! Lookup over the WAI-ARIA 1.2 role table in [`super::role_data`].

use super::attributes::{Scope, attribute};
use super::role_data::ROLES;

/// ARIA in HTML accepts `image` (a WAI-ARIA 1.3 name) for `img`; see C3 in
/// `standards/overlap.md`.
pub const ROLE_SYNONYMS: &[(&str, &str)] = &[("image", "img")];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameFrom {
    Author,
    Contents,
    Prohibited,
}

/// One entry of a role's "Required Owned Elements": `role`, owned directly,
/// or through an owned `via` element (e.g. `option` through a `group`).
#[derive(Debug)]
pub struct OwnedElement {
    pub role: &'static str,
    pub via: Option<&'static str>,
}

#[derive(Debug)]
pub struct Role {
    pub name: &'static str,
    pub is_abstract: bool,
    pub superclasses: &'static [&'static str],
    /// Every superclass, transitively.
    pub ancestors: &'static [&'static str],
    pub required_context: &'static [&'static str],
    pub required_owned: &'static [OwnedElement],
    pub name_from: &'static [NameFrom],
    pub children_presentational: bool,
    /// Supported non-global states and properties, including inherited ones;
    /// [`Role::supports`] adds the global ones.
    pub specific_attributes: &'static [&'static str],
    pub prohibited_attributes: &'static [&'static str],
    /// Supported, but deprecated on this role in WAI-ARIA 1.2.
    pub deprecated_attributes: &'static [&'static str],
    /// "Implicit Value for Role", including inherited ones: a required
    /// attribute with a default here need not be set.
    pub default_values: &'static [(&'static str, &'static str)],
}

impl Role {
    pub fn supports(&self, attribute: &str) -> bool {
        self.specific_attributes.contains(&attribute)
            || (is_global(attribute) && !self.prohibits(attribute))
    }

    pub fn prohibits(&self, attribute: &str) -> bool {
        self.prohibited_attributes.contains(&attribute)
    }

    pub fn deprecates(&self, attribute: &str) -> bool {
        self.deprecated_attributes.contains(&attribute)
    }

    pub fn default_value(&self, attribute: &str) -> Option<&'static str> {
        self.default_values
            .iter()
            .find(|(name, _)| *name == attribute)
            .map(|(_, value)| *value)
    }

    /// Whether this role is `ancestor` or one of its subclasses.
    pub fn is_a(&self, ancestor: &str) -> bool {
        self.name == ancestor || self.ancestors.contains(&ancestor)
    }

    pub fn is_presentational(&self) -> bool {
        matches!(self.name, "none" | "presentation")
    }
}

/// The role named `name` (lowercase), abstract ones included; synonyms from
/// [`ROLE_SYNONYMS`] resolve to the role they stand for.
pub fn role(name: &str) -> Option<&'static Role> {
    let name = ROLE_SYNONYMS
        .iter()
        .find(|(synonym, _)| *synonym == name)
        .map_or(name, |(_, canonical)| canonical);
    ROLES.iter().find(|role| role.name == name)
}

pub fn roles() -> impl Iterator<Item = &'static Role> {
    ROLES.iter()
}

/// A non-abstract WAI-ARIA 1.2 role, or a synonym of one: what a `role`
/// token may name.
pub fn concrete_role(name: &str) -> Option<&'static Role> {
    role(name).filter(|role| !role.is_abstract)
}

fn is_global(name: &str) -> bool {
    attribute(name).is_some_and(|attribute| attribute.scope != Scope::RoleSpecific)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> &'static Role {
        role(name).unwrap()
    }

    #[test]
    fn there_are_82_concrete_roles_and_12_abstract_ones() {
        assert_eq!(roles().filter(|role| !role.is_abstract).count(), 82);
        assert_eq!(roles().filter(|role| role.is_abstract).count(), 12);
    }

    #[test]
    fn abstract_roles_are_not_concrete() {
        assert!(role("widget").is_some());
        assert!(concrete_role("widget").is_none());
        assert!(concrete_role("button").is_some());
    }

    #[test]
    fn image_is_a_synonym_of_img() {
        assert_eq!(named("image").name, "img");
    }

    #[test]
    fn unknown_and_uppercase_names_are_not_roles() {
        assert!(role("buton").is_none());
        assert!(role("Button").is_none());
    }

    #[test]
    fn default_values_cover_some_required_attributes() {
        assert_eq!(
            named("option").default_value("aria-selected"),
            Some("false")
        );
        assert_eq!(
            named("treeitem").default_value("aria-selected"),
            Some("false")
        );
        assert_eq!(named("slider").default_value("aria-valuemax"), Some("100"));
        assert_eq!(named("checkbox").default_value("aria-checked"), None);
    }

    #[test]
    fn supports_role_specific_and_global_attributes() {
        let button = named("button");
        assert!(button.supports("aria-pressed"));
        assert!(button.supports("aria-label"));
        assert!(!button.supports("aria-checked"));
        assert!(!button.supports("aria-bogus"));
    }

    #[test]
    fn prohibited_attributes_are_not_supported() {
        let generic = named("generic");
        assert!(generic.prohibits("aria-label"));
        assert!(!generic.supports("aria-label"));
        assert!(generic.supports("aria-describedby"));
    }

    #[test]
    fn none_repeats_presentation() {
        assert!(named("none").prohibits("aria-labelledby"));
        assert!(named("none").is_presentational());
        assert_eq!(named("none").name_from, [NameFrom::Prohibited]);
    }

    #[test]
    fn global_deprecation_depends_on_the_role() {
        assert!(named("heading").deprecates("aria-disabled"));
        assert!(!named("button").deprecates("aria-disabled"));
        assert!(named("button").deprecates("aria-grabbed"));
    }

    #[test]
    fn subclass_roles_are_their_superclasses() {
        assert!(named("switch").is_a("checkbox"));
        assert!(named("banner").is_a("landmark"));
        assert!(!named("checkbox").is_a("switch"));
    }

    #[test]
    fn context_and_owned_elements() {
        assert_eq!(named("option").required_context, ["group", "listbox"]);
        let listbox_owns: Vec<_> = named("listbox")
            .required_owned
            .iter()
            .map(|owned| (owned.role, owned.via))
            .collect();
        assert_eq!(listbox_owns, [("option", Some("group")), ("option", None)]);
    }

    #[test]
    fn naming_characteristics() {
        assert!(named("button").children_presentational);
        assert_eq!(named("generic").name_from, [NameFrom::Prohibited]);
    }

    #[test]
    fn every_referenced_role_and_attribute_exists() {
        for role in roles() {
            let referenced_roles = role
                .superclasses
                .iter()
                .chain(role.ancestors)
                .chain(role.required_context)
                .chain(
                    role.required_owned
                        .iter()
                        .flat_map(|o| o.via.iter().chain([&o.role])),
                );
            for name in referenced_roles {
                assert!(super::role(name).is_some(), "{}: {name}", role.name);
            }
            let referenced_attributes = role
                .specific_attributes
                .iter()
                .chain(role.prohibited_attributes)
                .chain(role.deprecated_attributes)
                .chain(role.default_values.iter().map(|(name, _)| name));
            for name in referenced_attributes {
                assert!(attribute(name).is_some(), "{}: {name}", role.name);
            }
        }
    }
}
