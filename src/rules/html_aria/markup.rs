//! Quoting the offending markup in finding messages.

use crate::page::ElementRef;

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

/// `name="value"`, or just `name` when the value is empty.
pub fn attribute(el: ElementRef, name: &str) -> Option<String> {
    el.attr(name).map(|value| match value {
        "" => name.to_string(),
        value => format!("{name}=\"{value}\""),
    })
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
}
