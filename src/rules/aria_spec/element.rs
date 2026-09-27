//! Role, focus and visibility of a rendered element, as WAI-ARIA 1.2 and
//! ARIA in HTML define them. Everything here reads markup only: litehtml
//! exposes no computed `display`, runs no script, and has no focus state.

use crate::page::ElementRef;

use super::attributes::attribute;
use super::html::{implicit_role, input_type, is_details_summary};
use super::roles::{Role, concrete_role};
use super::values::{Parsed, is_aria_true, parse_value};

/// Elements the user agent stylesheet never renders.
const UNRENDERED_TAGS: &[&str] = &[
    "base", "head", "link", "meta", "script", "style", "template", "title",
];

/// The `role` attribute's tokens, lowercased: HTML matches role values
/// ASCII case-insensitively.
pub fn explicit_roles(el: ElementRef) -> Vec<String> {
    el.attr("role")
        .map(|value| {
            value
                .split_ascii_whitespace()
                .map(str::to_ascii_lowercase)
                .collect()
        })
        .unwrap_or_default()
}

/// The first `role` token naming a non-abstract WAI-ARIA 1.2 role (or the
/// `image` synonym, resolved to `img`); later tokens are fallbacks.
/// DPub-ARIA and Graphics-ARIA roles aren't WAI-ARIA 1.2 roles, so they're
/// skipped.
pub fn first_valid_role(el: ElementRef) -> Option<&'static Role> {
    explicit_roles(el)
        .iter()
        .find_map(|token| concrete_role(token))
}

/// The role the element is exposed with: its first valid explicit role,
/// else its implicit one. `none`/`presentation` give way to the implicit
/// role on a focusable element or one with a global `aria-*` attribute,
/// as WAI-ARIA's presentational role conflict resolution requires.
pub fn effective_role(el: ElementRef) -> Option<&'static Role> {
    match first_valid_role(el) {
        Some(role) if role.is_presentational() && overrides_presentation(el) => implicit_role(el),
        Some(role) => Some(role),
        None => implicit_role(el),
    }
}

/// The element's `aria-*` attributes, whatever their names.
pub fn aria_attributes<'a>(el: ElementRef<'a>) -> impl Iterator<Item = (&'a str, &'a str)> {
    el.node()
        .attrs
        .iter()
        .filter(|(name, _)| name.starts_with("aria-"))
        .map(|(name, value)| (name.as_str(), value.as_str()))
}

/// A global WAI-ARIA 1.2 state or property with a non-empty value.
fn has_global_aria_attribute(el: ElementRef) -> bool {
    aria_attributes(el).any(|(name, value)| {
        attribute(name).is_some_and(|attribute| {
            attribute.is_global() && parse_value(attribute, value) != Parsed::Empty
        })
    })
}

/// HTML's parsed `tabindex`: leading whitespace and trailing garbage are
/// ignored (`" 2px"` is 2), no digits is no tabindex.
pub fn tabindex(el: ElementRef) -> Option<i64> {
    let value = el
        .attr("tabindex")?
        .trim_start_matches(|c: char| c.is_ascii_whitespace());
    let (sign, rest) = match value.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, value.strip_prefix('+').unwrap_or(value)),
    };
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse::<i64>().ok().map(|number| sign * number)
}

/// Focusable by keyboard or pointer, `tabindex="-1"` included; see
/// [`is_tabbable`] for sequential focus.
pub fn is_focusable(el: ElementRef) -> bool {
    !is_not_rendered(el)
        && !is_inert(el)
        && !is_disabled(el)
        && (tabindex(el).is_some() || is_natively_focusable(el) || is_editing_host(el))
}

pub fn is_tabbable(el: ElementRef) -> bool {
    is_focusable(el) && tabindex(el).is_none_or(|index| index >= 0)
}

/// Hidden from assistive technologies: not rendered, `inert`, or
/// `aria-hidden="true"` on the element or an ancestor.
pub fn is_hidden(el: ElementRef) -> bool {
    is_not_rendered(el) || is_inert(el) || is_aria_hidden(el)
}

pub fn is_aria_hidden(el: ElementRef) -> bool {
    self_and_ancestors(el).any(|el| is_aria_true(el, "aria-hidden"))
}

/// Not rendered because of its markup alone: a `hidden` attribute, an
/// element the user agent stylesheet hides, `input type=hidden`, or the
/// content of a closed `dialog` or `details`. CSS `display: none` isn't
/// known.
pub fn is_not_rendered(el: ElementRef) -> bool {
    (el.tag() == "input" && input_type(el) == "hidden")
        || self_and_ancestors(el).any(|el| {
            el.has_attr("hidden") || UNRENDERED_TAGS.contains(&el.tag()) || is_closed_dialog(el)
        })
        || is_in_closed_details(el)
}

fn is_inert(el: ElementRef) -> bool {
    self_and_ancestors(el).any(|el| el.has_attr("inert"))
}

/// Disabled form control, including through a disabled `fieldset` (except
/// inside its first `legend`).
pub fn is_disabled(el: ElementRef) -> bool {
    let is_form_control = matches!(
        el.tag(),
        "button" | "input" | "select" | "textarea" | "optgroup" | "option" | "fieldset"
    );
    is_form_control && (el.has_attr("disabled") || is_in_disabled_fieldset(el))
}

fn overrides_presentation(el: ElementRef) -> bool {
    is_focusable(el) || has_global_aria_attribute(el)
}

fn is_natively_focusable(el: ElementRef) -> bool {
    match el.tag() {
        "a" | "area" => el.has_attr("href"),
        "button" | "select" | "textarea" | "iframe" => true,
        "input" => input_type(el) != "hidden",
        "audio" | "video" => el.has_attr("controls"),
        "summary" => is_details_summary(el),
        _ => false,
    }
}

fn is_editing_host(el: ElementRef) -> bool {
    el.attr("contenteditable").is_some_and(|value| {
        let value = value.trim();
        value.is_empty()
            || value.eq_ignore_ascii_case("true")
            || value.eq_ignore_ascii_case("plaintext-only")
    })
}

fn is_in_disabled_fieldset(el: ElementRef) -> bool {
    let mut child = el;
    for ancestor in el.ancestors() {
        if ancestor.tag() == "fieldset"
            && ancestor.has_attr("disabled")
            && !is_first_legend_of(child, ancestor)
        {
            return true;
        }
        child = ancestor;
    }
    false
}

fn is_first_legend_of(child: ElementRef, fieldset: ElementRef) -> bool {
    child.tag() == "legend"
        && fieldset
            .children()
            .find(|sibling| sibling.tag() == "legend")
            .is_some_and(|legend| legend == child)
}

fn is_closed_dialog(el: ElementRef) -> bool {
    el.tag() == "dialog" && !el.has_attr("open")
}

/// Inside a closed `details` but not in its summary.
fn is_in_closed_details(el: ElementRef) -> bool {
    let mut child = el;
    for ancestor in el.ancestors() {
        if ancestor.tag() == "details" && !ancestor.has_attr("open") && !is_details_summary(child) {
            return true;
        }
        child = ancestor;
    }
    false
}

fn self_and_ancestors<'a>(el: ElementRef<'a>) -> impl Iterator<Item = ElementRef<'a>> {
    std::iter::successors(Some(el), |el| el.parent())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn first<'a>(page: &'a crate::page::RenderedPage, tag: &'a str) -> ElementRef<'a> {
        page.by_tag(tag).next().unwrap()
    }

    #[test]
    fn explicit_roles_are_lowercased_tokens() {
        let page = page_from_html(r#"<div role=" Button  LINK">x</div>"#);
        assert_eq!(explicit_roles(first(&page, "div")), ["button", "link"]);
    }

    #[rstest]
    #[case(r#"<div role="bogus button link">x</div>"#, Some("button"))]
    #[case(r#"<div role="widget">x</div>"#, None)]
    #[case(r#"<div role="doc-toc">x</div>"#, None)]
    #[case(r#"<div role="image">x</div>"#, Some("img"))]
    #[case("<div>x</div>", None)]
    fn first_valid_roles(#[case] html: &str, #[case] expected: Option<&str>) {
        let page = page_from_html(html);
        assert_eq!(
            first_valid_role(first(&page, "div")).map(|r| r.name),
            expected
        );
    }

    #[rstest]
    #[case(r#"<nav role="bogus">x</nav>"#, "nav", Some("navigation"))]
    #[case(r#"<nav role="menubar">x</nav>"#, "nav", Some("menubar"))]
    #[case(r#"<h2 role="presentation">x</h2>"#, "h2", Some("presentation"))]
    #[case(r#"<h2 role="none" tabindex="0">x</h2>"#, "h2", Some("heading"))]
    #[case(
        r#"<h2 role="none" aria-describedby="d">x</h2>"#,
        "h2",
        Some("heading")
    )]
    #[case(r#"<h2 role="none" aria-describedby="">x</h2>"#, "h2", Some("none"))]
    #[case(r#"<button role="presentation">x</button>"#, "button", Some("button"))]
    fn effective_roles(#[case] html: &str, #[case] tag: &str, #[case] expected: Option<&str>) {
        let page = page_from_html(html);
        assert_eq!(effective_role(first(&page, tag)).map(|r| r.name), expected);
    }

    #[rstest]
    #[case(r#"<div tabindex="0">x</div>"#, Some(0))]
    #[case(r#"<div tabindex=" -1">x</div>"#, Some(-1))]
    #[case(r#"<div tabindex="2px">x</div>"#, Some(2))]
    #[case(r#"<div tabindex="x">x</div>"#, None)]
    #[case("<div>x</div>", None)]
    fn parses_tabindex(#[case] html: &str, #[case] expected: Option<i64>) {
        let page = page_from_html(html);
        assert_eq!(tabindex(first(&page, "div")), expected);
    }

    #[rstest]
    #[case(r#"<a href="/">x</a>"#, "a", true)]
    #[case("<a>x</a>", "a", false)]
    #[case("<button>x</button>", "button", true)]
    #[case("<button disabled>x</button>", "button", false)]
    #[case("<fieldset disabled><input></fieldset>", "input", false)]
    #[case(
        "<fieldset disabled><legend><input></legend></fieldset>",
        "input",
        true
    )]
    #[case(r#"<input type="hidden">"#, "input", false)]
    #[case(r#"<div tabindex="-1">x</div>"#, "div", true)]
    #[case(r#"<div contenteditable>x</div>"#, "div", true)]
    #[case(r#"<div contenteditable="false">x</div>"#, "div", false)]
    #[case(r#"<div hidden><button>x</button></div>"#, "button", false)]
    #[case(r#"<div inert><button>x</button></div>"#, "button", false)]
    #[case(r#"<div aria-hidden="true"><button>x</button></div>"#, "button", true)]
    #[case(
        "<details><summary>s</summary><button>x</button></details>",
        "summary",
        true
    )]
    #[case(
        "<details><summary>s</summary><button>x</button></details>",
        "button",
        false
    )]
    #[case(
        "<details open><summary>s</summary><button>x</button></details>",
        "button",
        true
    )]
    #[case("<video controls></video>", "video", true)]
    fn focusability(#[case] html: &str, #[case] tag: &str, #[case] expected: bool) {
        let page = page_from_html(html);
        assert_eq!(is_focusable(first(&page, tag)), expected);
    }

    #[test]
    fn negative_tabindex_is_focusable_but_not_tabbable() {
        let page = page_from_html(r#"<button tabindex="-1">x</button>"#);
        assert!(is_focusable(first(&page, "button")));
        assert!(!is_tabbable(first(&page, "button")));
    }

    #[rstest]
    #[case(r#"<div aria-hidden="true"><span>x</span></div>"#, "span", true)]
    #[case(r#"<div aria-hidden="false"><span>x</span></div>"#, "span", false)]
    #[case(r#"<div hidden><span>x</span></div>"#, "span", true)]
    #[case(r#"<div inert><span>x</span></div>"#, "span", true)]
    #[case("<dialog><span>x</span></dialog>", "span", true)]
    #[case("<dialog open><span>x</span></dialog>", "span", false)]
    #[case("<span>x</span>", "span", false)]
    fn hidden_elements(#[case] html: &str, #[case] tag: &str, #[case] expected: bool) {
        let page = page_from_html(html);
        assert_eq!(is_hidden(first(&page, tag)), expected);
    }

    #[test]
    fn aria_attributes_are_enumerated_including_unknown_ones() {
        let page = page_from_html(r#"<div id="a" aria-labeledby="x" aria-live="polite">x</div>"#);
        let names: Vec<_> = aria_attributes(first(&page, "div"))
            .map(|(name, _)| name)
            .collect();
        assert_eq!(names, ["aria-labeledby", "aria-live"]);
    }
}
