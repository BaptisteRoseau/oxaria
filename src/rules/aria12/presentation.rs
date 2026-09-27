//! Presentational roles: `none`/`presentation` that user agents must
//! ignore (ARIA-PRES001), presentational images with alt text
//! (ARIA-PRES002), and content lost inside roles whose children are
//! presentational (ARIA-PRES003).

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    Parsed, Role, aria_attributes, attribute, first_valid_role, is_aria_hidden, is_focusable,
    is_hidden, parse_value,
};
use crate::rules::{CheckOptions, Finding};

use super::support::quoted;
use super::tree::{
    exposed_role, has_presentational_children_ancestor, presentational_children_role,
};

/// Document structure roles whose meaning is lost inside a role with
/// presentational children. Text-level roles (`strong`, `code`, ...) and
/// images are left out: they're common inside buttons and lose nothing.
const STRUCTURAL_ROLES: &[&str] = &[
    "article",
    "blockquote",
    "cell",
    "columnheader",
    "definition",
    "dialog",
    "alertdialog",
    "feed",
    "figure",
    "heading",
    "list",
    "listitem",
    "row",
    "rowgroup",
    "rowheader",
    "table",
    "term",
];

/// ARIA-PRES001: user agents ignore `none`/`presentation` on a focusable
/// element or one with a global ARIA attribute.
pub fn check_ignored_presentation(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_presentational(*el) && !is_aria_hidden(*el))
        .filter_map(|el| {
            let (reason, help) = presentation_conflict(el)?;
            let role = el.attr("role").unwrap_or_default().trim();
            Some(
                Finding::error(
                    "ARIA-PRES001",
                    format!(
                        "role=\"{role}\" is ignored on <{}> because {reason}",
                        el.tag()
                    ),
                )
                .at(el)
                .help(help),
            )
        })
        .collect()
}

fn is_presentational(el: ElementRef) -> bool {
    first_valid_role(el).is_some_and(Role::is_presentational)
}

fn presentation_conflict(el: ElementRef) -> Option<(String, String)> {
    if is_focusable(el) {
        return Some((
            "it is focusable".to_string(),
            "remove the role: a focusable element keeps its semantics (if it really is \
             decoration, remove it from the focus order too)"
                .to_string(),
        ));
    }
    let (name, value) = global_attribute(el)?;
    Some((
        format!("it has the global attribute {}", quoted(name, value)),
        format!("remove the role, or remove {name} if the element's semantics should go"),
    ))
}

/// A global attribute other than `aria-hidden`, which hides the element
/// anyway, so the conflict doesn't matter.
fn global_attribute<'a>(el: ElementRef<'a>) -> Option<(&'a str, &'a str)> {
    aria_attributes(el).find(|(name, value)| {
        *name != "aria-hidden"
            && attribute(name).is_some_and(|attribute| {
                attribute.is_global() && parse_value(attribute, value) != Parsed::Empty
            })
    })
}

/// ARIA-PRES002: a presentational image's alt text is never exposed.
pub fn check_presentational_image_alt(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.by_tag("img")
        .filter(|img| is_presentational(*img))
        .filter_map(|img| Some((img, img.attr("alt").filter(|alt| !alt.trim().is_empty())?)))
        .map(|(img, alt)| {
            Finding::error(
                "ARIA-PRES002",
                format!("presentational <img> has {}", quoted("alt", alt)),
            )
            .at(img)
            .help(
                "set alt=\"\" if the image is decoration; if it carries meaning, remove the \
                 presentational role so the alt text is exposed",
            )
        })
        .collect()
}

/// ARIA-PRES003: inside a role whose children are presentational,
/// structure is flattened to text and focusable content becomes a hidden
/// control.
pub fn check_content_in_presentational_children(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_hidden(*el) && !has_presentational_children_ancestor(*el))
        .filter_map(|el| Some((el, presentational_children_role(el)?)))
        .flat_map(|(el, role)| {
            let mut lost = Vec::new();
            collect_lost_content(el, &mut lost);
            lost.into_iter()
                .map(move |(child, reason)| lost_content_finding(child, &reason, role))
        })
        .collect()
}

/// Stops at each offending element: what it contains is reported with it.
fn collect_lost_content<'a>(el: ElementRef<'a>, lost: &mut Vec<(ElementRef<'a>, String)>) {
    let children = el
        .children()
        .filter(|child| !child.node().is_text() && !is_hidden(*child));
    for child in children {
        match lost_reason(child) {
            Some(reason) => lost.push((child, reason)),
            None => collect_lost_content(child, lost),
        }
    }
}

fn lost_reason(el: ElementRef) -> Option<String> {
    let role = exposed_role(el);
    match role {
        _ if is_focusable(el) => Some(format!("focusable <{}>", el.tag())),
        Some(role) if is_lost_role(role) => {
            Some(format!("<{}> with role \"{}\"", el.tag(), role.name))
        }
        _ => None,
    }
}

fn is_lost_role(role: &Role) -> bool {
    STRUCTURAL_ROLES.contains(&role.name) || role.is_a("widget") || role.is_a("landmark")
}

fn lost_content_finding(child: ElementRef, reason: &str, container: &Role) -> Finding {
    Finding::error(
        "ARIA-PRES003",
        format!(
            "{reason} is inside role \"{}\", whose children are presentational",
            container.name
        ),
    )
    .at(child)
    .help(format!(
        "move the <{}> out of the {}, and keep only text or images inside the {}",
        child.tag(),
        container.name,
        container.name
    ))
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
    #[case(
        r#"<a href="/checkout" role="presentation">Checkout</a>"#,
        "it is focusable"
    )]
    #[case(
        r#"<h1 role="presentation" aria-describedby="comment-1">Sample Content</h1>"#,
        "global attribute"
    )]
    #[case(r#"<div role="none" tabindex="-1">x</div>"#, "it is focusable")]
    fn ignored_presentation_is_flagged(#[case] html: &str, #[case] reason: &str) {
        let findings = run(check_ignored_presentation, html);
        assert_eq!(findings.len(), 1, "{html}");
        assert!(findings[0].message.contains(reason), "{findings:?}");
    }

    #[rstest]
    #[case(r#"<a href="/checkout">Checkout</a>"#)]
    #[case(
        r#"<ul role="tree" aria-label="Files"><li role="presentation">
              <a role="treeitem" aria-expanded="true" aria-selected="false">Node</a></li></ul>"#
    )]
    #[case(r#"<img src="x.png" alt="" role="presentation" aria-hidden="true">"#)]
    #[case(r#"<table role="presentation"><tr><td>Layout</td></tr></table>"#)]
    #[case(r#"<h1 role="none" aria-describedby="">x</h1>"#)]
    fn effective_presentation_is_not_flagged(#[case] html: &str) {
        assert!(run(check_ignored_presentation, html).is_empty(), "{html}");
    }

    #[test]
    fn presentational_image_with_alt_is_flagged() {
        let findings = run(
            check_presentational_image_alt,
            r#"<img src="q3-sales.png" role="presentation" alt="Sales grew 20% in Q3">"#,
        );
        assert_eq!(findings.len(), 1);
    }

    #[rstest]
    #[case(r#"<div role="img" aria-labelledby="caption"><img src="example.png" role="presentation" alt="">
              <p id="caption">A visible text caption labeling the image.</p></div>"#)]
    #[case(r#"<img src="a.png" alt="A chart">"#)]
    #[case(r#"<img src="a.png" role="none">"#)]
    fn presentational_image_without_alt_is_not_flagged(#[case] html: &str) {
        assert!(
            run(check_presentational_image_alt, html).is_empty(),
            "{html}"
        );
    }

    #[rstest]
    #[case(
        r#"<div role="button" tabindex="0"><h3>Pro plan</h3>
              <ul><li>100 GB storage</li><li>Priority support</li></ul></div>"#,
        2
    )]
    #[case(
        r#"<div role="tab" aria-selected="false"><a href="/details">Details</a></div>"#,
        1
    )]
    #[case(r#"<button><button>Inner</button></button>"#, 0)]
    #[case(
        r#"<div role="button"><div role="button"><a href="/x">x</a></div></div>"#,
        1
    )]
    fn lost_content_is_flagged(#[case] html: &str, #[case] count: usize) {
        assert_eq!(
            run(check_content_in_presentational_children, html).len(),
            count,
            "{html}"
        );
    }

    #[rstest]
    #[case(r#"<div role="button" tabindex="0">Choose the Pro plan</div>"#)]
    #[case(r#"<div role="tab" aria-selected="false">Details</div>"#)]
    #[case(r#"<button><img src="i.png" alt=""><strong>Save</strong> <svg aria-hidden="true"></svg></button>"#)]
    #[case(r#"<div role="img" aria-label="Chart"><img src="a.png" alt=""><img src="b.png" alt=""></div>"#)]
    #[case(r#"<button><span aria-hidden="true"><a href="/x">x</a></span></button>"#)]
    fn plain_content_is_not_flagged(#[case] html: &str) {
        assert!(
            run(check_content_in_presentational_children, html).is_empty(),
            "{html}"
        );
    }
}
