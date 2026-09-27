//! Role values authors must not use: abstract roles (ARIA-ROLE002) and an
//! explicit `generic` (ARIA-ROLE003).

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{explicit_roles, role};
use crate::rules::{CheckOptions, Finding};

/// Concrete roles to suggest for each abstract one.
const CONCRETE_ALTERNATIVES: &[(&str, &str)] = &[
    ("command", "button, link or menuitem"),
    (
        "composite",
        "grid, listbox, menu, radiogroup, tablist or tree",
    ),
    (
        "input",
        "checkbox, combobox, radio, slider, spinbutton or textbox",
    ),
    (
        "landmark",
        "banner, complementary, contentinfo, form, main, navigation, region or search",
    ),
    (
        "range",
        "meter, progressbar, scrollbar, slider or spinbutton",
    ),
    (
        "roletype",
        "a concrete role that describes the element, or no role",
    ),
    ("section", "article, group or region"),
    ("sectionhead", "heading"),
    ("select", "combobox, listbox, radiogroup or tree"),
    (
        "structure",
        "a concrete role that describes the element, or no role",
    ),
    (
        "widget",
        "button, checkbox, link, slider or another concrete widget role",
    ),
    ("window", "dialog or alertdialog"),
];

/// ARIA-ROLE002: abstract roles only organize the role taxonomy; assistive
/// technologies don't support them in content.
pub fn check_abstract_role(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all().flat_map(abstract_role_findings).collect()
}

fn abstract_role_findings(el: ElementRef) -> Vec<Finding> {
    explicit_roles(el)
        .into_iter()
        .filter(|token| role(token).is_some_and(|role| role.is_abstract))
        .map(|token| {
            Finding::error(
                "ARIA-ROLE002",
                format!("role=\"{token}\" is an abstract role, which content must not use"),
            )
            .at(el)
            .help(abstract_role_help(&token))
        })
        .collect()
}

fn abstract_role_help(token: &str) -> String {
    let alternatives = CONCRETE_ALTERNATIVES
        .iter()
        .find(|(abstract_role, _)| *abstract_role == token)
        .map_or("a concrete role", |(_, alternatives)| alternatives);
    format!("replace role=\"{token}\" with {alternatives}")
}

/// ARIA-ROLE003: `generic` exists for user agents to map elements such as
/// `div` and `span`; authors remove or group semantics with other roles.
pub fn check_explicit_generic(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| explicit_roles(*el).iter().any(|token| token == "generic"))
        .map(|el| {
            Finding::error(
                "ARIA-ROLE003",
                format!("<{}> sets role=\"generic\" explicitly", el.tag()),
            )
            .at(el)
            .help(
                "remove the role (a <div> or <span> is already generic), use role=\"none\" to \
                 remove the element's semantics, or a role such as \"group\" to group and name \
                 its content",
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn abstract_roles(html: &str) -> Vec<Finding> {
        check_abstract_role(&page_from_html(html), &CheckOptions::default())
    }

    fn generic_roles(html: &str) -> Vec<Finding> {
        check_explicit_generic(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(
        r#"<div role="landmark" aria-label="Site links">...</div>"#,
        "navigation"
    )]
    #[case(r#"<div role="range" aria-valuenow="40">40%</div>"#, "progressbar")]
    #[case(r#"<div role="button Widget">x</div>"#, "concrete widget")]
    fn abstract_roles_are_flagged(#[case] html: &str, #[case] suggestion: &str) {
        let findings = abstract_roles(html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].help.as_deref().unwrap().contains(suggestion));
    }

    #[rstest]
    #[case(r#"<div role="navigation" aria-label="Site links">...</div>"#)]
    #[case(r#"<div role="progressbar" aria-label="Upload" aria-valuenow="40">40%</div>"#)]
    #[case(r#"<div role="doc-toc">x</div>"#)]
    fn concrete_roles_are_not_flagged(#[case] html: &str) {
        assert!(abstract_roles(html).is_empty());
    }

    #[test]
    fn explicit_generic_is_flagged() {
        let findings = generic_roles(r#"<section role="generic" class="card">...</section>"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].message,
            "<section> sets role=\"generic\" explicitly"
        );
    }

    #[rstest]
    #[case(r#"<section role="none" class="card">...</section>"#)]
    #[case(r#"<div role="group" aria-label="Shipping options">...</div>"#)]
    #[case("<div>implicitly generic</div>")]
    fn other_roles_are_not_generic(#[case] html: &str) {
        assert!(generic_roles(html).is_empty());
    }
}
