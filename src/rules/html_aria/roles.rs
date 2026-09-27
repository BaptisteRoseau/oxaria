//! ARIA in HTML's requirements on `role` values: redundant, generic and
//! abstract roles.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    AriaValue, Parsed, Role, aria_value, explicit_roles, first_valid_role, implicit_role, role,
    roles,
};
use crate::rules::{CheckOptions, Finding};

use super::markup::start_tag;

const HEADINGS: &[(&str, i64)] = &[
    ("h1", 1),
    ("h2", 2),
    ("h3", 3),
    ("h4", 4),
    ("h5", 5),
    ("h6", 6),
];

const SUGGESTED_ROLE_COUNT: usize = 5;

/// HTMLARIA002: a `role` or `aria-level` repeating the element's implicit
/// semantics.
pub fn check_redundant_semantics(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| redundant_role(el).into_iter().chain(redundant_level(el)))
        .collect()
}

fn redundant_role(el: ElementRef) -> Option<Finding> {
    let explicit = canonical(first_valid_role(el)?.name);
    let implicit = canonical(implicit_role_name(el)?);
    let is_redundant = explicit == implicit && !is_accepted_redundancy(el, explicit);
    is_redundant.then(|| {
        let tag = start_tag(el, &["type", "role"]);
        Finding::error(
            "HTMLARIA002",
            format!("{tag} sets role \"{explicit}\", which the element already has implicitly"),
        )
        .at(el)
        .help(format!(
            "remove the role attribute: <{}> is already exposed as {explicit}",
            el.tag()
        ))
    })
}

fn implicit_role_name(el: ElementRef) -> Option<&'static str> {
    match is_details_summary(el) {
        true => Some("button"),
        false => implicit_role(el).map(|role| role.name),
    }
}

fn canonical(role: &str) -> &str {
    match role {
        "presentation" => "none",
        role => role,
    }
}

/// `generic` is HTMLARIA003's; `role="list"` restores list semantics
/// Safari drops from lists styled without markers.
fn is_accepted_redundancy(el: ElementRef, role: &str) -> bool {
    match role {
        "generic" => true,
        "list" => matches!(el.tag(), "menu" | "ol" | "ul"),
        _ => false,
    }
}

fn redundant_level(el: ElementRef) -> Option<Finding> {
    let level = heading_level(el)?;
    let keeps_heading_role = first_valid_role(el).is_none_or(|role| role.name == "heading");
    let is_redundant = keeps_heading_role
        && aria_value(el, "aria-level") == Some(Parsed::Valid(AriaValue::Integer(level)));
    is_redundant.then(|| {
        Finding::error(
            "HTMLARIA002",
            format!(
                "{} repeats the level <{}> already has",
                start_tag(el, &["aria-level"]),
                el.tag()
            ),
        )
        .at(el)
        .help(format!(
            "remove aria-level: <{}> is already a level {level} heading",
            el.tag()
        ))
    })
}

fn heading_level(el: ElementRef) -> Option<i64> {
    HEADINGS
        .iter()
        .find(|(tag, _)| *tag == el.tag())
        .map(|(_, level)| *level)
}

fn is_details_summary(el: ElementRef) -> bool {
    el.tag() == "summary"
        && el.parent().is_some_and(|parent| {
            parent.tag() == "details"
                && parent
                    .children()
                    .find(|child| child.tag() == "summary")
                    .is_some_and(|summary| summary == el)
        })
}

/// HTMLARIA003: `role="generic"` anywhere, or `generic`/`document` on
/// `html`.
pub fn check_generic_role(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter_map(|el| discouraged_role(el).map(|role| generic_role_finding(el, role)))
        .collect()
}

fn discouraged_role(el: ElementRef) -> Option<&'static str> {
    let role = first_valid_role(el)?.name;
    match (el.tag(), role) {
        (_, "generic") | ("html", "document") => Some(role),
        _ => None,
    }
}

fn generic_role_finding(el: ElementRef, role: &str) -> Finding {
    let help = match el.tag() {
        "html" => "remove the role attribute from <html>".to_string(),
        "div" | "span" => "remove role=\"generic\": the element is already generic".to_string(),
        tag => format!(
            "use a <div> (or <span>) instead of <{tag} role=\"generic\">, or role=\"none\" to \
             remove the <{tag}>'s semantics"
        ),
    };
    Finding::error(
        "HTMLARIA003",
        format!(
            "{} sets role \"{role}\", which authors should not use",
            start_tag(el, &["role"])
        ),
    )
    .at(el)
    .help(help)
}

/// HTMLARIA004: a `role` token naming an abstract WAI-ARIA role.
pub fn check_abstract_role(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            explicit_roles(el)
                .into_iter()
                .filter_map(|token| role(&token).filter(|role| role.is_abstract))
                .map(move |abstract_role| abstract_role_finding(el, abstract_role))
        })
        .collect()
}

fn abstract_role_finding(el: ElementRef, abstract_role: &Role) -> Finding {
    let name = abstract_role.name;
    Finding::error(
        "HTMLARIA004",
        format!(
            "{} uses the abstract role \"{name}\", which is only meant for the ARIA taxonomy",
            start_tag(el, &["role"])
        ),
    )
    .at(el)
    .help(format!(
        "replace \"{name}\" with a concrete role that fits the widget, such as {}",
        concrete_subclasses(name)
    ))
}

/// Direct concrete subclasses when there are any (`select` -> `combobox`,
/// `listbox`, ...), else concrete descendants.
fn concrete_subclasses(abstract_role: &str) -> String {
    let concrete = || roles().filter(|role| !role.is_abstract);
    let direct: Vec<_> = concrete()
        .filter(|role| role.superclasses.contains(&abstract_role))
        .collect();
    let candidates = match direct.is_empty() {
        true => concrete()
            .filter(|role| role.ancestors.contains(&abstract_role))
            .collect(),
        false => direct,
    };
    candidates
        .iter()
        .take(SUGGESTED_ROLE_COUNT)
        .map(|role| format!("\"{}\"", role.name))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(check: fn(&RenderedPage, &CheckOptions) -> Vec<Finding>, html: &str) -> Vec<Finding> {
        check(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(r#"<button role="button">x</button>"#)]
    #[case(r#"<details><summary role="button">more</summary>x</details>"#)]
    #[case(r#"<header role="banner">x</header>"#)]
    #[case(r#"<nav role="Navigation">x</nav>"#)]
    #[case(r#"<a href="/" role="link">x</a>"#)]
    #[case(r#"<input type="checkbox" role="checkbox">"#)]
    #[case(r#"<img src="a.png" alt="A cat" role="image">"#)]
    #[case(r#"<img src="a.png" alt="" role="presentation">"#)]
    #[case(r#"<section aria-label="News" role="region">x</section>"#)]
    #[case(r#"<h2 aria-level="2">x</h2>"#)]
    fn redundant_semantics_are_flagged(#[case] html: &str) {
        let findings = run(check_redundant_semantics, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA002");
    }

    #[rstest]
    #[case("<button>x</button>")]
    #[case("<details><summary>more</summary>x</details>")]
    #[case(r#"<article><header role="banner">x</header></article>"#)]
    #[case(r#"<section role="region">x</section>"#)]
    #[case(r#"<ul role="list"><li>x</li></ul>"#)]
    #[case(r#"<div role="generic">x</div>"#)]
    #[case(r#"<summary role="button">x</summary>"#)]
    #[case(r#"<h2 aria-level="3">x</h2>"#)]
    #[case(r#"<div role="heading" aria-level="2">x</div>"#)]
    #[case(r#"<h2 role="tab" aria-level="2">x</h2>"#)]
    #[case(r#"<a role="link">x</a>"#)]
    fn meaningful_semantics_are_not_flagged(#[case] html: &str) {
        assert!(run(check_redundant_semantics, html).is_empty());
    }

    #[test]
    fn redundant_role_help_names_the_role() {
        let findings = run(
            check_redundant_semantics,
            r#"<header role="banner">x</header>"#,
        );
        assert_eq!(
            findings[0].message,
            r#"<header role="banner"> sets role "banner", which the element already has implicitly"#
        );
        assert_eq!(
            findings[0].help.as_deref(),
            Some("remove the role attribute: <header> is already exposed as banner")
        );
    }

    #[rstest]
    #[case(r#"<article role="generic">x</article>"#)]
    #[case(r#"<div role="GENERIC">x</div>"#)]
    #[case(r#"<html role="document"><body>x</body></html>"#)]
    fn generic_roles_are_flagged(#[case] html: &str) {
        let findings = run(check_generic_role, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA003");
    }

    #[rstest]
    #[case("<div>x</div>")]
    #[case(r#"<div role="none">x</div>"#)]
    #[case(r#"<div role="document">x</div>"#)]
    #[case(r#"<div role="button generic">x</div>"#)]
    fn other_roles_are_not_generic(#[case] html: &str) {
        assert!(run(check_generic_role, html).is_empty());
    }

    #[test]
    fn generic_help_suggests_a_div() {
        let findings = run(check_generic_role, r#"<article role="generic">x</article>"#);
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with("use a <div>")
        );
    }

    #[rstest]
    #[case(r#"<div role="select">x</div>"#)]
    #[case(r#"<div role="Widget">x</div>"#)]
    #[case(r#"<div role="combobox landmark">x</div>"#)]
    fn abstract_roles_are_flagged(#[case] html: &str) {
        let findings = run(check_abstract_role, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA004");
    }

    #[rstest]
    #[case(r#"<div role="combobox">x</div>"#)]
    #[case(r#"<div role="doc-toc">x</div>"#)]
    #[case(r#"<div role="bogus">x</div>"#)]
    fn concrete_roles_are_not_flagged(#[case] html: &str) {
        assert!(run(check_abstract_role, html).is_empty());
    }

    #[test]
    fn abstract_role_help_suggests_concrete_subclasses() {
        let findings = run(check_abstract_role, r#"<div role="select">x</div>"#);
        let help = findings[0].help.as_deref().unwrap();
        assert!(help.contains("\"radiogroup\""), "{help}");
        assert!(help.contains("\"listbox\""), "{help}");
    }

    #[test]
    fn roletype_suggests_concrete_descendants() {
        let findings = run(check_abstract_role, r#"<div role="roletype">x</div>"#);
        assert!(findings[0].help.as_deref().unwrap().contains("such as \""));
    }
}
