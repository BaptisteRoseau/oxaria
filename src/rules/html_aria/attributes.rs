//! ARIA in HTML's requirements on `aria-*` attributes: which ones each
//! element allows, `aria-hidden`, and those the native element handles.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    AllowedAria, AriaValue, Parsed, Role, allowed_aria, aria_attributes, attribute, effective_role,
    explicit_roles, first_valid_role, input_type, is_aria_true, is_focusable, parse_value,
    role as named_role, roles, tabindex,
};
use crate::rules::{CheckOptions, Finding};

use super::markup::start_tag;

/// Governed by ARIA in HTML's separate naming requirement (HTMLARIA008),
/// which isn't implemented: see `standards/overlap.md` C6.
const NAMING_ATTRIBUTES: &[&str] = &["aria-label", "aria-labelledby"];

const SUGGESTED_ROLE_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Disallowed {
    NoAria,
    AriaHiddenOnly,
    AriaHiddenTrueOnly,
    GlobalOnly,
    Prohibited(&'static str),
    Unsupported(&'static str),
}

/// HTMLARIA007: `aria-*` attributes the element (or its role) doesn't
/// allow. Unknown attribute names are left to WAI-ARIA's own rules.
pub fn check_disallowed_aria(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            aria_attributes(el)
                .filter_map(move |(name, value)| {
                    disallowed(el, name, value).map(|reason| disallowed_finding(el, name, reason))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn disallowed(el: ElementRef, name: &str, value: &str) -> Option<Disallowed> {
    let attribute = attribute(name)?;
    match allowed_aria(el) {
        AllowedAria::NoAria => Some(Disallowed::NoAria),
        AllowedAria::AriaHiddenOnly => {
            (name != "aria-hidden").then_some(Disallowed::AriaHiddenOnly)
        }
        AllowedAria::AriaHiddenTrueOnly => {
            let is_hidden_true = name == "aria-hidden"
                && parse_value(attribute, value) == Parsed::Valid(AriaValue::True);
            (!is_hidden_true).then_some(Disallowed::AriaHiddenTrueOnly)
        }
        AllowedAria::GlobalOnly => (!attribute.is_global()).then_some(Disallowed::GlobalOnly),
        AllowedAria::GlobalAnd(extra) => {
            (!attribute.is_global() && !extra.contains(&name)).then_some(Disallowed::GlobalOnly)
        }
        AllowedAria::GlobalAndRole(role_name) => role_disallows(el, name, named_role(role_name)),
        AllowedAria::GlobalAndEffectiveRole if has_unknown_semantics(el) => None,
        AllowedAria::GlobalAndEffectiveRole => role_disallows(el, name, effective_role(el)),
    }
}

/// A custom element may get its role from `ElementInternals` (script),
/// and DPub/Graphics roles aren't in the WAI-ARIA 1.2 tables.
fn has_unknown_semantics(el: ElementRef) -> bool {
    let is_custom_element = el.tag().contains('-') && first_valid_role(el).is_none();
    let has_module_role = explicit_roles(el)
        .first()
        .is_some_and(|token| token.starts_with("doc-") || token.starts_with("graphics-"));
    is_custom_element || has_module_role
}

fn role_disallows(el: ElementRef, name: &str, role: Option<&'static Role>) -> Option<Disallowed> {
    let is_left_to_other_rules =
        NAMING_ATTRIBUTES.contains(&name) || (el.tag() == "body" && name == "aria-hidden");
    if is_left_to_other_rules {
        return None;
    }
    match role {
        Some(role) if role.prohibits(name) => Some(Disallowed::Prohibited(role.name)),
        Some(role) if !role.supports(name) => Some(Disallowed::Unsupported(role.name)),
        Some(_) => None,
        None => {
            let is_global = attribute(name).is_some_and(|attribute| attribute.is_global());
            (!is_global).then_some(Disallowed::GlobalOnly)
        }
    }
}

fn disallowed_finding(el: ElementRef, name: &str, reason: Disallowed) -> Finding {
    Finding::error(
        "HTMLARIA007",
        format!(
            "{} doesn't allow {name}",
            start_tag(el, &["type", "alt", "role", name])
        ),
    )
    .at(el)
    .help(disallowed_help(el, name, reason))
}

fn disallowed_help(el: ElementRef, name: &str, reason: Disallowed) -> String {
    let tag = el.tag();
    match reason {
        Disallowed::NoAria => format!("remove {name}: <{tag}> takes no aria-* attributes"),
        Disallowed::AriaHiddenOnly => format!("remove {name}: <{tag}> only takes aria-hidden"),
        Disallowed::AriaHiddenTrueOnly => format!(
            "remove {name}: a decorative <img alt=\"\"> only takes aria-hidden=\"true\"; give it \
             a real alt text instead if it conveys information"
        ),
        Disallowed::GlobalOnly => format!(
            "remove {name}: <{tag}> only takes global aria-* attributes{}",
            supporting_roles(name)
        ),
        Disallowed::Prohibited(role) => format!("remove {name}: the {role} role prohibits it"),
        Disallowed::Unsupported(role) => format!(
            "remove {name}: the {role} role of <{tag}> doesn't support it{}",
            supporting_roles(name)
        ),
    }
}

fn supporting_roles(name: &str) -> String {
    let supporting: Vec<_> = roles()
        .filter(|role| !role.is_abstract && role.specific_attributes.contains(&name))
        .take(SUGGESTED_ROLE_COUNT)
        .map(|role| role.name)
        .collect();
    match supporting.is_empty() {
        true => String::new(),
        false => format!(" (roles such as {} do)", supporting.join(", ")),
    }
}

/// HTMLARIA009: `aria-hidden="true"` on `body` or on a focusable element.
pub fn check_hidden_focusable(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_aria_true(*el, "aria-hidden"))
        .filter_map(|el| match el.tag() {
            "body" => Some(hidden_body_finding(el)),
            _ if is_focusable(el) => Some(hidden_focusable_finding(el)),
            _ => None,
        })
        .collect()
}

fn hidden_body_finding(body: ElementRef) -> Finding {
    Finding::error(
        "HTMLARIA009",
        "<body aria-hidden=\"true\"> hides the whole page from assistive technologies".to_string(),
    )
    .at(body)
    .help(
        "remove aria-hidden from <body>; to hide the page behind a modal dialog, use inert on \
         the content outside the dialog instead",
    )
}

fn hidden_focusable_finding(el: ElementRef) -> Finding {
    let help = match tabindex(el) {
        Some(_) => "remove aria-hidden, or remove tabindex too if the element shouldn't be \
                    focused"
            .to_string(),
        None => format!(
            "remove aria-hidden, or hide the <{}> from everyone with the hidden or inert \
             attribute",
            el.tag()
        ),
    };
    Finding::error(
        "HTMLARIA009",
        format!(
            "{} can receive focus but is hidden from assistive technologies",
            start_tag(el, &["type", "tabindex", "aria-hidden"])
        ),
    )
    .at(el)
    .help(help)
}

/// HTMLARIA010: `aria-hidden="true"` repeating the `hidden` attribute.
pub fn check_hidden_twice(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| el.has_attr("hidden") && is_aria_true(*el, "aria-hidden"))
        .map(hidden_twice_finding)
        .collect()
}

fn hidden_twice_finding(el: ElementRef) -> Finding {
    let until_found = el
        .attr("hidden")
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("until-found"));
    let help = match until_found {
        true => {
            "remove aria-hidden=\"true\": it would keep the content hidden from assistive \
                 technologies after find-in-page reveals it"
        }
        false => {
            "remove aria-hidden=\"true\": the hidden attribute already hides the element \
                  from everyone"
        }
    };
    Finding::error(
        "HTMLARIA010",
        format!(
            "{} sets aria-hidden on an element that already has the hidden attribute",
            start_tag(el, &["hidden", "aria-hidden"])
        ),
    )
    .at(el)
    .help(help)
}

/// An `aria-*` attribute the native element's own row advises against.
struct Advice {
    attribute: &'static str,
    applies: fn(ElementRef) -> bool,
    help: &'static str,
}

const ADVICE: &[Advice] = &[
    Advice {
        attribute: "aria-disabled",
        applies: is_disabled_link,
        help: "to disable the link, remove href and keep role=\"link\" aria-disabled=\"true\"; \
               otherwise remove aria-disabled",
    },
    Advice {
        attribute: "aria-haspopup",
        applies: is_input_with_suggestions,
        help: "remove aria-haspopup: an <input list> already exposes its suggestions popup",
    },
    Advice {
        attribute: "aria-selected",
        applies: is_option,
        help: "remove aria-selected and use the selected attribute",
    },
    Advice {
        attribute: "aria-multiselectable",
        applies: is_select,
        help: "remove aria-multiselectable and use the multiple attribute",
    },
];

/// HTMLARIA011: `aria-*` attributes that native elements handle
/// themselves.
pub fn check_advised_against_aria(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            ADVICE
                .iter()
                .filter(move |advice| el.has_attr(advice.attribute) && (advice.applies)(el))
                .map(move |advice| advised_against_finding(el, advice))
        })
        .collect()
}

fn advised_against_finding(el: ElementRef, advice: &Advice) -> Finding {
    Finding::error(
        "HTMLARIA011",
        format!(
            "{} uses {}, which <{}> should not use",
            start_tag(el, &["type", "list", advice.attribute]),
            advice.attribute,
            el.tag()
        ),
    )
    .at(el)
    .help(advice.help)
}

fn is_disabled_link(el: ElementRef) -> bool {
    el.tag() == "a"
        && el.has_attr("href")
        && is_aria_true(el, "aria-disabled")
        && effective_role(el).is_some_and(|role| role.name == "link")
}

fn is_input_with_suggestions(el: ElementRef) -> bool {
    el.tag() == "input"
        && el.has_attr("list")
        && matches!(input_type(el), "text" | "search" | "tel" | "url" | "email")
}

fn is_option(el: ElementRef) -> bool {
    el.tag() == "option"
}

fn is_select(el: ElementRef) -> bool {
    el.tag() == "select"
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
    #[case(r#"<br aria-label="line break">"#)]
    #[case(r#"<input type="file" aria-valuenow="3">"#)]
    #[case(r#"<script aria-hidden="true"></script>"#)]
    #[case(r#"<input type="hidden" aria-hidden="true">"#)]
    #[case(r#"<img src="a.png" alt="" aria-describedby="d">"#)]
    #[case(r#"<img src="a.png" alt="" aria-hidden="false">"#)]
    #[case(r#"<legend aria-checked="true">x</legend>"#)]
    #[case(r#"<button aria-checked="true">x</button>"#)]
    #[case(r#"<div aria-expanded="false">x</div>"#)]
    #[case(r#"<span aria-roledescription="thing">x</span>"#)]
    #[case(r#"<input type="color" aria-required="true">"#)]
    #[case(r#"<dl><dd aria-checked="true">x</dd></dl>"#)]
    #[case(r#"<input type="password" aria-checked="true">"#)]
    #[case(r#"<abbr aria-pressed="true">x</abbr>"#)]
    #[case(r#"<details><summary aria-expanded="true">s</summary></details>"#)]
    fn disallowed_attributes_are_flagged(#[case] html: &str) {
        let findings = run(check_disallowed_aria, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA007");
    }

    #[rstest]
    #[case(r#"<br aria-hidden="true">"#)]
    #[case(r#"<input type="file" aria-required="true">"#)]
    #[case(r#"<img src="a.png" alt="" aria-hidden="true">"#)]
    #[case(r#"<img src="a.png" alt="A cat" aria-describedby="d">"#)]
    #[case(r#"<button aria-pressed="true" aria-label="Bold">B</button>"#)]
    #[case(r#"<a href="/" aria-expanded="false">x</a>"#)]
    #[case(r#"<div role="checkbox" aria-checked="true">x</div>"#)]
    #[case(r#"<div aria-label="x">x</div>"#)]
    #[case(r#"<div aria-live="polite" aria-bogus="x" aria-description="d">x</div>"#)]
    #[case(r#"<input type="password" aria-required="true">"#)]
    #[case(r#"<input type="checkbox" aria-invalid="true">"#)]
    #[case(r#"<my-switch aria-checked="true">x</my-switch>"#)]
    #[case(r#"<a href="/" role="doc-noteref" aria-expanded="true">1</a>"#)]
    #[case(r#"<details><summary aria-haspopup="true">s</summary></details>"#)]
    #[case(r#"<body aria-hidden="true"></body>"#)]
    #[case(r#"<h2 aria-level="2" aria-disabled="true">x</h2>"#)]
    fn allowed_attributes_are_not_flagged(#[case] html: &str) {
        assert!(run(check_disallowed_aria, html).is_empty());
    }

    #[test]
    fn disallowed_help_explains_the_limit() {
        let findings = run(
            check_disallowed_aria,
            r#"<button aria-checked="true">x</button>"#,
        );
        assert_eq!(
            findings[0].message,
            r#"<button aria-checked="true"> doesn't allow aria-checked"#
        );
        let help = findings[0].help.as_deref().unwrap();
        assert!(
            help.starts_with("remove aria-checked: the button role of <button> doesn't support"),
            "{help}"
        );
        assert!(help.contains("checkbox"), "{help}");
    }

    #[test]
    fn prohibited_attributes_name_the_prohibiting_role() {
        let findings = run(
            check_disallowed_aria,
            r#"<span aria-roledescription="thing">x</span>"#,
        );
        assert_eq!(
            findings[0].help.as_deref(),
            Some("remove aria-roledescription: the generic role prohibits it")
        );
    }

    #[rstest]
    #[case(r#"<body aria-hidden="true"><p>x</p></body>"#)]
    #[case(r#"<button aria-hidden="true">Close</button>"#)]
    #[case(r#"<div tabindex="-1" aria-hidden="true">x</div>"#)]
    #[case(r#"<a href="/" aria-hidden="true">x</a>"#)]
    fn hidden_focusable_elements_are_flagged(#[case] html: &str) {
        let findings = run(check_hidden_focusable, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA009");
    }

    #[rstest]
    #[case("<body><p>x</p></body>")]
    #[case("<button>Close</button>")]
    #[case(r#"<div tabindex="-1">x</div>"#)]
    #[case(r#"<button aria-hidden="false">x</button>"#)]
    #[case(r#"<button disabled aria-hidden="true">x</button>"#)]
    #[case(r#"<div aria-hidden="true"><span>x</span></div>"#)]
    #[case(r#"<a aria-hidden="true">x</a>"#)]
    fn unfocusable_or_visible_elements_are_not_flagged(#[case] html: &str) {
        assert!(run(check_hidden_focusable, html).is_empty());
    }

    #[test]
    fn hidden_focusable_help_depends_on_tabindex() {
        let findings = run(
            check_hidden_focusable,
            r#"<div tabindex="-1" aria-hidden="true">x</div>"#,
        );
        assert!(findings[0].help.as_deref().unwrap().contains("tabindex"));
    }

    #[rstest]
    #[case(
        r#"<div hidden="until-found" aria-hidden="true">x</div>"#,
        "find-in-page"
    )]
    #[case(r#"<div hidden aria-hidden="true">x</div>"#, "already hides")]
    fn hidden_twice_is_flagged(#[case] html: &str, #[case] help: &str) {
        let findings = run(check_hidden_twice, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA010");
        assert!(findings[0].help.as_deref().unwrap().contains(help));
    }

    #[rstest]
    #[case(r#"<div hidden="until-found">x</div>"#)]
    #[case(r#"<div aria-hidden="true">x</div>"#)]
    #[case(r#"<div hidden aria-hidden="false">x</div>"#)]
    fn single_hiding_is_not_flagged(#[case] html: &str) {
        assert!(run(check_hidden_twice, html).is_empty());
    }

    #[rstest]
    #[case(r#"<a href="/archive" aria-disabled="true">Archive</a>"#)]
    #[case(r#"<select aria-multiselectable="true"><option>a</option></select>"#)]
    #[case(r#"<select><option aria-selected="true">a</option></select>"#)]
    #[case(r#"<input list="l" aria-haspopup="listbox">"#)]
    #[case(r#"<input type="email" list="l" aria-haspopup="true">"#)]
    fn advised_against_attributes_are_flagged(#[case] html: &str) {
        let findings = run(check_advised_against_aria, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA011");
    }

    #[rstest]
    #[case(r#"<a role="link" aria-disabled="true">Archive</a>"#)]
    #[case(r#"<a href="/" aria-disabled="false">x</a>"#)]
    #[case(r#"<a href="/" role="button" aria-disabled="true">x</a>"#)]
    #[case("<select multiple><option>a</option></select>")]
    #[case(r#"<div role="option" aria-selected="true">a</div>"#)]
    #[case(r#"<input aria-haspopup="true">"#)]
    #[case(r#"<input type="number" list="l" aria-haspopup="true">"#)]
    fn native_friendly_attributes_are_not_flagged(#[case] html: &str) {
        assert!(run(check_advised_against_aria, html).is_empty());
    }
}
