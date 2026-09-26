//! The plain, owned snapshot every rule queries: a rendered page's DOM
//! structure together with its computed color, font, and layout data.
//!
//! [`RenderedPage`] is an arena of [`RenderedElement`]s connected by
//! `parent`/`children` indices rather than borrowed references, so it is
//! trivially `Send + Sync` and can be shared via `Arc` across parallel rule
//! checks without the lifetime/thread-safety constraints a live DOM handle
//! (from litehtml or `scraper`) would carry.

use url::Url;

pub const TEXT_TAG: &str = "#text";

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn smallest_dimension(&self) -> f32 {
        self.width.min(self.height)
    }
}

#[derive(Debug, Clone)]
pub struct RenderedElement {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub own_text: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub color: (u8, u8, u8),
    pub background_color: Option<(u8, u8, u8)>,
    pub font_size_px: f32,
    pub font_weight: u32,
    pub bounding_box: Rect,
}

impl RenderedElement {
    pub fn is_text(&self) -> bool {
        self.tag == TEXT_TAG
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderedPage {
    pub elements: Vec<RenderedElement>,
    /// Raw text content of every `<style>` block in the source document, in
    /// document order. Extracted directly from the source HTML text rather
    /// than through the render tree, since G195 is the one rule that needs
    /// literal, unrendered CSS source text (litehtml has no `:focus` state
    /// simulation, so that check stays a source-level heuristic).
    pub stylesheets: Vec<String>,
    /// Where the page was fetched from; `None` for a local file. Lets rules
    /// tell a same-site absolute link (`https://site/pricing`) from a
    /// relative one (`/pricing`) pointing at the same place.
    pub url: Option<Url>,
}

impl RenderedPage {
    pub fn get(&self, index: usize) -> ElementRef<'_> {
        ElementRef { page: self, index }
    }

    pub fn all(&self) -> impl Iterator<Item = ElementRef<'_>> {
        (0..self.elements.len())
            .map(|index| self.get(index))
            .filter(|el| !el.node().is_text())
    }

    pub fn select(&self, predicate: impl Fn(ElementRef) -> bool) -> Vec<ElementRef<'_>> {
        self.all().filter(|el| predicate(*el)).collect()
    }

    pub fn by_tag<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = ElementRef<'a>> {
        self.all().filter(move |el| el.tag() == tag)
    }

    pub fn element_by_id(&self, id: &str) -> Option<ElementRef<'_>> {
        self.all().find(|el| el.attr("id") == Some(id))
    }

    pub fn stylesheet_text(&self) -> String {
        self.stylesheets.join("\n")
    }
}

#[derive(Clone, Copy)]
pub struct ElementRef<'a> {
    page: &'a RenderedPage,
    index: usize,
}

impl<'a> ElementRef<'a> {
    pub fn node(&self) -> &'a RenderedElement {
        &self.page.elements[self.index]
    }

    pub fn tag(&self) -> &'a str {
        &self.node().tag
    }

    pub fn attr(&self, name: &str) -> Option<&'a str> {
        self.node()
            .attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.attr(name).is_some()
    }

    pub fn color(&self) -> (u8, u8, u8) {
        self.node().color
    }

    pub fn background_color(&self) -> Option<(u8, u8, u8)> {
        self.node().background_color
    }

    pub fn font_size_px(&self) -> f32 {
        self.node().font_size_px
    }

    pub fn font_weight(&self) -> u32 {
        self.node().font_weight
    }

    pub fn bounding_box(&self) -> Rect {
        self.node().bounding_box
    }

    pub fn parent(&self) -> Option<ElementRef<'a>> {
        self.node().parent.map(|index| self.page.get(index))
    }

    pub fn ancestors(&self) -> impl Iterator<Item = ElementRef<'a>> {
        std::iter::successors(self.parent(), |el| el.parent())
    }

    pub fn children(&self) -> impl Iterator<Item = ElementRef<'a>> + 'a {
        let page = self.page;
        self.node()
            .children
            .clone()
            .into_iter()
            .map(move |index| page.get(index))
    }

    pub fn descendants(&self) -> impl Iterator<Item = ElementRef<'a>> {
        let mut stack: Vec<usize> = self.node().children.iter().rev().copied().collect();
        let page = self.page;
        std::iter::from_fn(move || {
            let index = stack.pop()?;
            stack.extend(page.elements[index].children.iter().rev());
            Some(page.get(index))
        })
    }

    /// A CSS-selector-like path locating this element in the document,
    /// e.g. `body > main > div:nth-of-type(2) > button`. It stops at the
    /// nearest element with a unique id (`button#submit`), which keeps the
    /// path short and stable when unrelated parts of the page change.
    pub fn selector(&self) -> String {
        let mut segments = Vec::new();
        for el in std::iter::once(*self).chain(self.ancestors()) {
            if el.tag().is_empty() || el.tag() == "html" {
                continue;
            }
            if let Some(id) = el.attr("id").filter(|id| self.is_unique_id(id)) {
                segments.push(format!("{}#{id}", el.tag()));
                break;
            }
            segments.push(el.selector_segment());
        }
        segments.reverse();
        segments.join(" > ")
    }

    /// Only ids usable as-is in a CSS selector; others fall back to the
    /// structural path rather than needing CSS escaping.
    fn is_unique_id(&self, id: &str) -> bool {
        let is_plain = id.starts_with(|c: char| c.is_ascii_alphabetic())
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        is_plain
            && self
                .page
                .all()
                .filter(|el| el.attr("id") == Some(id))
                .take(2)
                .count()
                == 1
    }

    fn selector_segment(&self) -> String {
        let same_tag_siblings: Vec<usize> = match self.parent() {
            Some(parent) => parent
                .children()
                .filter(|sibling| sibling.tag() == self.tag())
                .map(|sibling| sibling.index)
                .collect(),
            None => vec![self.index],
        };
        match same_tag_siblings.len() {
            1 => self.tag().to_string(),
            _ => {
                let position = same_tag_siblings.iter().position(|&i| i == self.index);
                format!("{}:nth-of-type({})", self.tag(), position.unwrap_or(0) + 1)
            }
        }
    }

    /// Recursive, whitespace-normalized text content of this element and its
    /// descendants (mirroring `scraper::ElementRef::text()` + collapsing).
    pub fn text(&self) -> String {
        let mut buf = String::new();
        collect_text(*self, &mut buf);
        buf.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

fn collect_text(el: ElementRef, buf: &mut String) {
    if el.node().is_text() {
        buf.push_str(&el.node().own_text);
        buf.push(' ');
        return;
    }
    for child in el.children() {
        collect_text(child, buf);
    }
}

/// A simplified subset of the WAI-ARIA accessible-name computation:
/// `aria-labelledby`, then `aria-label`, then an associated `label` (via
/// `for` or wrapping), then visible text content, then `alt`, `value` (for
/// submit/button inputs), then `title`.
pub fn accessible_name(page: &RenderedPage, el: ElementRef) -> String {
    if let Some(name) = name_from_labelledby(page, el) {
        return name;
    }
    if let Some(name) = non_empty_attr(el, "aria-label") {
        return name;
    }
    if let Some(name) = name_from_label(page, el) {
        return name;
    }
    let text = el.text();
    if !text.is_empty() {
        return text;
    }
    if let Some(name) = non_empty_attr(el, "alt") {
        return name;
    }
    if is_submit_or_button_input(el)
        && let Some(name) = non_empty_attr(el, "value")
    {
        return name;
    }
    non_empty_attr(el, "title").unwrap_or_default()
}

fn name_from_labelledby(page: &RenderedPage, el: ElementRef) -> Option<String> {
    let ids = el.attr("aria-labelledby")?;
    let text = ids_text(page, ids);
    (!text.is_empty()).then_some(text)
}

pub fn ids_text(page: &RenderedPage, ids: &str) -> String {
    ids.split_whitespace()
        .filter_map(|id| page.element_by_id(id))
        .map(|el| el.text())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn name_from_label<'a>(page: &'a RenderedPage, el: ElementRef<'a>) -> Option<String> {
    let label = label_for(page, el).or_else(|| label_ancestor(el))?;
    let text = label.text();
    (!text.is_empty()).then_some(text)
}

fn label_for<'a>(page: &'a RenderedPage, el: ElementRef<'a>) -> Option<ElementRef<'a>> {
    let id = el.attr("id")?;
    page.by_tag("label")
        .find(|label| label.attr("for") == Some(id))
}

fn label_ancestor(el: ElementRef) -> Option<ElementRef> {
    el.ancestors().find(|ancestor| ancestor.tag() == "label")
}

fn non_empty_attr(el: ElementRef, name: &str) -> Option<String> {
    el.attr(name)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn is_submit_or_button_input(el: ElementRef) -> bool {
    el.tag() == "input" && matches!(el.attr("type"), Some("submit") | Some("button"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn select_finds_matching_elements() {
        let p = page_from_html(r#"<a href="/a"></a><a href="/b"></a>"#);
        assert_eq!(p.by_tag("a").count(), 2);
    }

    #[test]
    fn element_by_id_finds_match() {
        let p = page_from_html(r#"<div id="main">Content</div>"#);
        assert!(p.element_by_id("main").is_some());
    }

    #[test]
    fn element_by_id_returns_none_when_absent() {
        let p = page_from_html("<p>Nothing</p>");
        assert!(p.element_by_id("missing").is_none());
    }

    #[test]
    fn text_collapses_whitespace_across_text_nodes() {
        let p = page_from_html("<p>  Hello   \n  world  </p>");
        assert_eq!(p.by_tag("p").next().unwrap().text(), "Hello world");
    }

    #[test]
    fn accessible_name_prefers_labelledby() {
        let p = page_from_html(
            r#"<span id="lbl">Billing address</span><input aria-labelledby="lbl" value="ignored">"#,
        );
        let input = p.by_tag("input").next().unwrap();
        assert_eq!(accessible_name(&p, input), "Billing address");
    }

    #[test]
    fn accessible_name_falls_back_to_aria_label() {
        let p = page_from_html(r#"<button aria-label="Close dialog"></button>"#);
        let button = p.by_tag("button").next().unwrap();
        assert_eq!(accessible_name(&p, button), "Close dialog");
    }

    #[test]
    fn accessible_name_falls_back_to_text_content() {
        let p = page_from_html("<button>Submit</button>");
        let button = p.by_tag("button").next().unwrap();
        assert_eq!(accessible_name(&p, button), "Submit");
    }

    #[test]
    fn accessible_name_uses_label_for_text() {
        let p = page_from_html(r#"<label for="a">Email</label><input id="a">"#);
        let input = p.by_tag("input").next().unwrap();
        assert_eq!(accessible_name(&p, input), "Email");
    }

    #[test]
    fn accessible_name_uses_wrapping_label_text() {
        let p = page_from_html(r#"<label>Email <input id="a"></label>"#);
        let input = p.by_tag("input").next().unwrap();
        assert_eq!(accessible_name(&p, input), "Email");
    }

    #[test]
    fn accessible_name_falls_back_to_value_for_submit_input() {
        let p = page_from_html(r#"<input type="submit" value="Send it">"#);
        let input = p.by_tag("input").next().unwrap();
        assert_eq!(accessible_name(&p, input), "Send it");
    }

    #[test]
    fn accessible_name_is_empty_when_nothing_found() {
        let p = page_from_html("<button></button>");
        let button = p.by_tag("button").next().unwrap();
        assert_eq!(accessible_name(&p, button), "");
    }

    #[test]
    fn ids_text_joins_multiple_ids() {
        let p = page_from_html(r#"<span id="a">Hello</span><span id="b">World</span>"#);
        assert_eq!(ids_text(&p, "a b"), "Hello World");
    }

    #[test]
    fn descendants_visits_nested_children_in_document_order() {
        let p = page_from_html("<div><span><b></b></span><em></em><i></i></div>");
        let div = p.by_tag("div").next().unwrap();
        let tags: Vec<&str> = div
            .descendants()
            .map(|el| el.tag())
            .filter(|t| *t != TEXT_TAG)
            .collect();
        assert_eq!(tags, vec!["span", "b", "em", "i"]);
    }

    fn selector_of(html: &str, tag: &str, nth: usize) -> String {
        let page = page_from_html(html);
        let el = page.by_tag(tag).nth(nth).unwrap();
        el.selector()
    }

    #[test]
    fn selector_is_a_structural_path_from_body() {
        let html = "<body><main><div><p>a</p></div><div><button>x</button></div></main></body>";
        assert_eq!(
            selector_of(html, "button", 0),
            "body > main > div:nth-of-type(2) > button"
        );
    }

    #[test]
    fn selector_counts_only_same_tag_siblings() {
        let html = "<body><div><span>a</span><button>1</button><span>b</span><button>2</button></div></body>";
        assert_eq!(
            selector_of(html, "button", 1),
            "body > div > button:nth-of-type(2)"
        );
    }

    #[test]
    fn selector_stops_at_the_nearest_unique_id() {
        let html = r#"<body><div id="nav"><ul><li><a href="/">a</a></li></ul></div></body>"#;
        assert_eq!(selector_of(html, "a", 0), "div#nav > ul > li > a");
    }

    #[test]
    fn selector_uses_the_elements_own_unique_id() {
        let html = r#"<body><div><button id="submit">x</button></div></body>"#;
        assert_eq!(selector_of(html, "button", 0), "button#submit");
    }

    #[test]
    fn selector_ignores_duplicate_and_unusual_ids() {
        let html = r#"<body><div id="d"><b id="a:b">x</b></div><div id="d"></div></body>"#;
        assert_eq!(selector_of(html, "b", 0), "body > div:nth-of-type(1) > b");
    }
}
