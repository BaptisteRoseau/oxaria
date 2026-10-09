//! Quoting the offending markup in finding messages.

use crate::page::ElementRef;

const MAX_QUOTED_VALUE: usize = 40;

/// The element's start tag with only the named attributes it has, in the
/// given order, e.g. `<input type="checkbox" checked>`.
pub fn start_tag(el: ElementRef, names: &[&str]) -> String {
    let attributes: String = names
        .iter()
        .filter_map(|name| attribute(el, name))
        .map(|attribute| format!(" {attribute}"))
        .collect();
    format!("<{}{attributes}>", el.tag())
}

/// The element's start tag with only the first named attribute it has,
/// always as `name="value"`, e.g. `<a href="">`.
pub fn start_tag_first(el: ElementRef, names: &[&str]) -> String {
    match names.iter().find_map(|name| Some((name, el.attr(name)?))) {
        Some((name, value)) => format!("<{} {name}=\"{value}\">", el.tag()),
        None => format!("<{}>", el.tag()),
    }
}

/// `name="value"`, or just `name` when the value is empty.
pub fn attribute(el: ElementRef, name: &str) -> Option<String> {
    el.attr(name).map(|value| match value {
        "" => name.to_string(),
        value => format!("{name}=\"{value}\""),
    })
}

/// `name="value"`, with a long value shortened.
pub fn quoted(name: &str, value: &str) -> String {
    format!("{name}=\"{}\"", shortened(value))
}

pub fn shortened(value: &str) -> String {
    let value = value.trim();
    match value.char_indices().nth(MAX_QUOTED_VALUE) {
        Some((end, _)) => format!("{}...", &value[..end]),
        None => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn quotes_only_the_named_attributes_in_order() {
        let page = page_from_html(r#"<input id="a" checked type="checkbox">"#);
        let input = page.by_tag("input").next().unwrap();
        assert_eq!(
            start_tag(input, &["type", "checked", "role"]),
            r#"<input type="checkbox" checked>"#
        );
    }

    #[test]
    fn quotes_only_the_first_named_attribute_even_when_empty() {
        let page = page_from_html(r#"<a role="" href="/x">x</a><b>y</b>"#);
        let link = page.by_tag("a").next().unwrap();
        assert_eq!(
            start_tag_first(link, &["type", "role", "href"]),
            r#"<a role="">"#
        );
        let bold = page.by_tag("b").next().unwrap();
        assert_eq!(start_tag_first(bold, &["href"]), "<b>");
    }

    #[test]
    fn long_values_are_shortened() {
        assert_eq!(quoted("aria-label", " Close "), r#"aria-label="Close""#);
        let long = "x".repeat(50);
        assert_eq!(
            quoted("aria-label", &long),
            format!("aria-label=\"{}...\"", "x".repeat(40))
        );
    }
}
