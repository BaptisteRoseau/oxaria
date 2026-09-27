//! The accessible name computation (accname 1.2 with HTML-AAM's host
//! language steps), as far as the ACT naming rules need it. Unlike
//! [`ElementRef::accessible_name`], it follows `aria-labelledby` into its
//! targets' own names, skips hidden content, only names an element from its
//! content when its role allows it, and reads the `title` of an `svg` and
//! the `aria-label` of a descendant.

use crate::page::ElementRef;
use crate::rules::aria_spec::{NameFrom, effective_role, input_type, is_hidden};

/// Elements a `label` can name, `input type=image` excepted: HTML-AAM
/// names it from `alt` instead.
const LABELABLE_TAGS: &[&str] = &[
    "button", "input", "meter", "output", "progress", "select", "textarea",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Traversal {
    Root,
    Content,
    /// Inside an `aria-labelledby` target: hidden content counts, and
    /// nested `aria-labelledby` isn't followed.
    Labelledby,
}

pub fn accessible_name(el: ElementRef) -> String {
    name(el, Traversal::Root).unwrap_or_default()
}

fn name(el: ElementRef, traversal: Traversal) -> Option<String> {
    if traversal == Traversal::Content && is_hidden(el) {
        return None;
    }
    labelledby_name(el, traversal)
        .or_else(|| non_empty(el.attr("aria-label")))
        .or_else(|| native_name(el))
        .or_else(|| content_name(el, traversal))
        .or_else(|| non_empty(el.attr("title")))
        .or_else(|| placeholder(el))
}

fn labelledby_name(el: ElementRef, traversal: Traversal) -> Option<String> {
    if traversal == Traversal::Labelledby {
        return None;
    }
    let names: Vec<String> = el
        .attr("aria-labelledby")?
        .split_ascii_whitespace()
        .filter_map(|id| el.page().element_by_id(id))
        .filter_map(|target| name(target, Traversal::Labelledby))
        .collect();
    non_empty(Some(&names.join(" ")))
}

fn native_name(el: ElementRef) -> Option<String> {
    match el.tag() {
        "img" | "area" => non_empty(el.attr("alt")),
        "input" => input_name(el),
        "svg" => svg_title(el),
        tag if LABELABLE_TAGS.contains(&tag) => el.label_text(),
        _ if is_in_svg(el) => svg_title(el),
        _ => None,
    }
}

fn input_name(input: ElementRef) -> Option<String> {
    match input_type(input) {
        "image" => non_empty(input.attr("alt")),
        "submit" => input_label_or_value(input).or_else(|| Some("Submit".to_string())),
        "reset" => input_label_or_value(input).or_else(|| Some("Reset".to_string())),
        "button" => input_label_or_value(input),
        _ => input.label_text(),
    }
}

fn input_label_or_value(input: ElementRef) -> Option<String> {
    input
        .label_text()
        .or_else(|| non_empty(input.attr("value")))
}

fn svg_title(el: ElementRef) -> Option<String> {
    el.children()
        .find(|child| child.tag() == "title")
        .and_then(|title| non_empty(Some(&title.text())))
}

fn content_name(el: ElementRef, traversal: Traversal) -> Option<String> {
    if traversal == Traversal::Root && !is_named_from_content(el) {
        return None;
    }
    let inner = match traversal {
        Traversal::Labelledby => Traversal::Labelledby,
        _ => Traversal::Content,
    };
    let parts: Vec<String> = el
        .children()
        .filter_map(|child| match child.node().is_text() {
            true => Some(child.node().own_text.clone()),
            false => name(child, inner),
        })
        .collect();
    non_empty(Some(&parts.join(" ")))
}

/// `summary` has no implicit role in ARIA in HTML, but is named from its
/// content like the button it is exposed as.
fn is_named_from_content(el: ElementRef) -> bool {
    el.tag() == "summary"
        || effective_role(el).is_some_and(|role| role.name_from.contains(&NameFrom::Contents))
}

fn placeholder(el: ElementRef) -> Option<String> {
    match el.tag() {
        "input" | "textarea" => non_empty(el.attr("placeholder")),
        _ => None,
    }
}

fn is_in_svg(el: ElementRef) -> bool {
    el.ancestors().any(|ancestor| ancestor.tag() == "svg")
}

fn non_empty(value: Option<&str>) -> Option<String> {
    let normalized = value?.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn name_of(html: &str, tag: &str) -> String {
        let page = page_from_html(html);
        let el = page.by_tag(tag).next().unwrap();
        accessible_name(el)
    }

    #[rstest]
    #[case(r#"<a href="/"> Web  Accessibility </a>"#, "a", "Web Accessibility")]
    #[case(r#"<a href="/"><img src="l.png" alt=""></a>"#, "a", "")]
    #[case(r#"<a href="/"><img src="l.png" title="WAI"></a>"#, "a", "WAI")]
    #[case(
        r#"<a href="/"><img src="l.png" aria-labelledby="x"></a><div id="x">WAI</div>"#,
        "a",
        "WAI"
    )]
    #[case(
        r#"<a href="/"><svg role="img" aria-label="Home"></svg></a>"#,
        "a",
        "Home"
    )]
    #[case(r#"<a href="/"><svg><title>Home</title></svg></a>"#, "a", "Home")]
    #[case(r#"<a href="/"><span aria-hidden="true">x</span></a>"#, "a", "")]
    #[case(
        r#"<a href="/" title="Home"><img src="l.png" alt=""></a>"#,
        "a",
        "Home"
    )]
    #[case(r#"<button type="button" value="read more"></button>"#, "button", "")]
    #[case(r#"<input type="submit">"#, "input", "Submit")]
    #[case(r#"<input type="reset">"#, "input", "Reset")]
    #[case(r#"<input type="button">"#, "input", "")]
    #[case(r#"<input type="image" name="search" src="s.svg">"#, "input", "")]
    #[case(r#"<input placeholder="Search">"#, "input", "Search")]
    #[case(r#"<label>first name <input></label>"#, "input", "first name")]
    #[case(r#"<label>first name <div role="textbox"></div></label>"#, "div", "")]
    #[case(r#"<div role="textbox">first name</div>"#, "div", "")]
    #[case(r#"<div role="checkbox">I agree</div>"#, "div", "I agree")]
    #[case(r#"<select id="c"><option>England</option></select>"#, "select", "")]
    #[case(
        r#"<input aria-labelledby="k"><span id="k" aria-hidden="true">Ketchup</span>"#,
        "input",
        "Ketchup"
    )]
    #[case(
        r#"<input aria-labelledby="k"><span id="k" aria-label="Mustard"></span>"#,
        "input",
        "Mustard"
    )]
    #[case(r#"<div role="img">Some text</div>"#, "div", "")]
    #[case(
        r#"<details><summary>Opening times</summary></details>"#,
        "summary",
        "Opening times"
    )]
    fn computes_names(#[case] html: &str, #[case] tag: &str, #[case] expected: &str) {
        assert_eq!(name_of(html, tag), expected, "{html}");
    }

    #[test]
    fn labelledby_does_not_recurse() {
        let html = r#"<input aria-labelledby="a"><span id="a" aria-labelledby="b"></span><span id="b">B</span>"#;
        assert_eq!(name_of(html, "input"), "");
    }
}
