//! The plain, owned snapshot every rule queries: a rendered page's DOM
//! structure together with its computed color, font, and layout data, as an
//! index-linked arena so it is `Send + Sync`.

use url::Url;

pub const TEXT_TAG: &str = "#text";
pub const DEFAULT_COLOR: (u8, u8, u8) = (0, 0, 0);
pub const DEFAULT_FONT_SIZE_PX: f32 = 16.0;
pub const DEFAULT_FONT_WEIGHT: u32 = 400;

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
    pub fn new(tag: String, parent: Option<usize>) -> Self {
        RenderedElement {
            tag,
            attrs: Vec::new(),
            own_text: String::new(),
            parent,
            children: Vec::new(),
            color: DEFAULT_COLOR,
            background_color: None,
            font_size_px: DEFAULT_FONT_SIZE_PX,
            font_weight: DEFAULT_FONT_WEIGHT,
            bounding_box: Rect::default(),
        }
    }

    pub fn is_text(&self) -> bool {
        self.tag == TEXT_TAG
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderedPage {
    pub elements: Vec<RenderedElement>,
    /// Raw text of every `<style>` block, in document order, taken from the
    /// source HTML rather than the render tree.
    pub stylesheets: Vec<String>,
    /// Where the page was served from (after redirects); `None` for a local file. Lets rules
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
            .filter(|el| !el.is_text())
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

    pub fn ids_text(&self, ids: &str) -> String {
        ids.split_whitespace()
            .filter_map(|id| self.element_by_id(id))
            .map(|el| el.text())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}

#[derive(Clone, Copy)]
pub struct ElementRef<'a> {
    page: &'a RenderedPage,
    index: usize,
}

impl PartialEq for ElementRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.page, other.page) && self.index == other.index
    }
}

impl<'a> ElementRef<'a> {
    pub fn node(&self) -> &'a RenderedElement {
        &self.page.elements[self.index]
    }

    pub fn page(&self) -> &'a RenderedPage {
        self.page
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

    /// The raw value, when it isn't empty or only whitespace.
    pub fn non_empty_attr(&self, name: &str) -> Option<&'a str> {
        self.attr(name).filter(|value| !value.trim().is_empty())
    }

    pub fn is_text(&self) -> bool {
        self.node().is_text()
    }

    /// A child text node with more than whitespace.
    pub fn has_own_text(&self) -> bool {
        self.children()
            .any(|child| child.is_text() && !child.node().own_text.trim().is_empty())
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

    pub fn self_and_ancestors(&self) -> impl Iterator<Item = ElementRef<'a>> {
        std::iter::successors(Some(*self), |el| el.parent())
    }

    /// Each ancestor paired with its child on the path from this element.
    pub fn ancestors_with_child(&self) -> impl Iterator<Item = (ElementRef<'a>, ElementRef<'a>)> {
        self.self_and_ancestors()
            .filter_map(|child| child.parent().map(|ancestor| (child, ancestor)))
    }

    /// The nearest ancestor with this tag, never the element itself.
    pub fn nearest_ancestor(&self, tag: &str) -> Option<ElementRef<'a>> {
        self.ancestors().find(|ancestor| ancestor.tag() == tag)
    }

    /// Whether an ancestor, not the element itself, has this tag.
    pub fn has_ancestor(&self, tag: &str) -> bool {
        self.nearest_ancestor(tag).is_some()
    }

    pub fn children(&self) -> impl Iterator<Item = ElementRef<'a>> + 'a {
        let page = self.page;
        self.node()
            .children
            .iter()
            .map(move |&index| page.get(index))
    }

    pub fn element_children(&self) -> impl Iterator<Item = ElementRef<'a>> + 'a {
        self.children().filter(|child| !child.is_text())
    }

    pub fn first_child(&self, tag: &str) -> Option<ElementRef<'a>> {
        self.children().find(|child| child.tag() == tag)
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
        for el in self.self_and_ancestors() {
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

    /// A simplified subset of the WAI-ARIA accessible-name computation:
    /// `aria-labelledby`, then `aria-label`, then an associated `label` (via
    /// `for` or wrapping), then visible text content, then the `alt` of images
    /// it contains, then its own `alt`, `value` (for submit/button inputs), then
    /// `title`.
    pub fn accessible_name(&self) -> String {
        let el = *self;
        if let Some(name) = name_from_labelledby(el) {
            return name;
        }
        if let Some(name) = trimmed_attr(el, "aria-label") {
            return name;
        }
        if let Some(name) = name_from_label(el) {
            return name;
        }
        let text = el.text();
        if !text.is_empty() {
            return text;
        }
        if let Some(name) = name_from_images(el) {
            return name;
        }
        if let Some(name) = trimmed_attr(el, "alt") {
            return name;
        }
        if is_submit_or_button_input(el)
            && let Some(name) = trimmed_attr(el, "value")
        {
            return name;
        }
        trimmed_attr(el, "title").unwrap_or_default()
    }

    /// Text of the `label` associated with this control (via `for` or wrapping), if any.
    pub fn label_text(&self) -> Option<String> {
        name_from_label(*self)
    }
}

fn collect_text(el: ElementRef, buf: &mut String) {
    if el.is_text() {
        buf.push_str(&el.node().own_text);
        buf.push(' ');
        return;
    }
    for child in el.children() {
        collect_text(child, buf);
    }
}

fn name_from_labelledby(el: ElementRef) -> Option<String> {
    let ids = el.attr("aria-labelledby")?;
    let text = el.page.ids_text(ids);
    (!text.is_empty()).then_some(text)
}

fn name_from_images(el: ElementRef) -> Option<String> {
    let alts: Vec<String> = el
        .descendants()
        .filter(|descendant| descendant.tag() == "img")
        .filter_map(|img| trimmed_attr(img, "alt"))
        .collect();
    (!alts.is_empty()).then(|| alts.join(" "))
}

fn name_from_label(el: ElementRef) -> Option<String> {
    let label = label_for(el).or_else(|| el.nearest_ancestor("label"))?;
    let text = label.text();
    (!text.is_empty()).then_some(text)
}

fn label_for(el: ElementRef) -> Option<ElementRef> {
    let id = el.attr("id")?;
    el.page
        .by_tag("label")
        .find(|label| label.attr("for") == Some(id))
}

fn trimmed_attr(el: ElementRef, name: &str) -> Option<String> {
    el.non_empty_attr(name)
        .map(|value| value.trim().to_string())
}

fn is_submit_or_button_input(el: ElementRef) -> bool {
    el.tag() == "input" && matches!(el.attr("type"), Some("submit" | "button"))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

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

    #[rstest]
    // aria-labelledby is preferred.
    #[case(
        r#"<span id="lbl">Billing address</span><input aria-labelledby="lbl" value="ignored">"#,
        "input",
        "Billing address"
    )]
    // Then aria-label.
    #[case(
        r#"<button aria-label="Close dialog"></button>"#,
        "button",
        "Close dialog"
    )]
    // Then text content.
    #[case("<button>Submit</button>", "button", "Submit")]
    #[case(r#"<label for="a">Email</label><input id="a">"#, "input", "Email")]
    #[case(r#"<label>Email <input id="a"></label>"#, "input", "Email")]
    // The alt of contained images.
    #[case(
        r#"<a href="/"><img src="a.png" alt="Acme home page"></a>"#,
        "a",
        "Acme home page"
    )]
    // Then the value of a submit input.
    #[case(r#"<input type="submit" value="Send it">"#, "input", "Send it")]
    #[case("<button></button>", "button", "")]
    fn accessible_name(#[case] html: &str, #[case] tag: &str, #[case] expected: &str) {
        let p = page_from_html(html);
        assert_eq!(p.by_tag(tag).next().unwrap().accessible_name(), expected);
    }

    #[test]
    fn ids_text_joins_multiple_ids() {
        let p = page_from_html(r#"<span id="a">Hello</span><span id="b">World</span>"#);
        assert_eq!(p.ids_text("a b"), "Hello World");
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

    #[test]
    fn self_and_ancestors_starts_with_the_element() {
        let p = page_from_html("<div><p><b>x</b></p></div>");
        let tags: Vec<&str> = p
            .by_tag("b")
            .next()
            .unwrap()
            .self_and_ancestors()
            .map(|el| el.tag())
            .collect();
        assert_eq!(tags, vec!["b", "p", "div", "body", "html"]);
    }

    #[test]
    fn ancestors_with_child_pairs_each_ancestor_with_the_child_it_was_reached_from() {
        let p = page_from_html("<div><p><b>x</b></p></div>");
        let pairs: Vec<(&str, &str)> = p
            .by_tag("b")
            .next()
            .unwrap()
            .ancestors_with_child()
            .map(|(child, ancestor)| (child.tag(), ancestor.tag()))
            .collect();
        assert_eq!(
            pairs,
            vec![("b", "p"), ("p", "div"), ("div", "body"), ("body", "html")]
        );
    }

    #[rstest]
    #[case("p", "div", Some("inner"))]
    // The element itself never matches.
    #[case("#inner", "div", Some("outer"))]
    #[case("p", "p", None)]
    fn nearest_ancestor(#[case] from: &str, #[case] tag: &str, #[case] expected: Option<&str>) {
        let p = page_from_html(r#"<div id="outer"><div id="inner"><p>x</p></div></div>"#);
        let el = match from.strip_prefix('#') {
            Some(id) => p.element_by_id(id),
            None => p.by_tag(from).next(),
        }
        .unwrap();
        assert_eq!(
            el.nearest_ancestor(tag).and_then(|a| a.attr("id")),
            expected
        );
        assert_eq!(el.has_ancestor(tag), expected.is_some());
    }

    #[test]
    fn element_children_and_first_child_skip_text() {
        let p = page_from_html("<div>a<b>1</b>b<i></i><b>2</b></div>");
        let div = p.by_tag("div").next().unwrap();
        let tags: Vec<&str> = div.element_children().map(|el| el.tag()).collect();
        assert_eq!(tags, vec!["b", "i", "b"]);
        assert_eq!(
            div.first_child("b").map(|b| b.text()),
            Some("1".to_string())
        );
        assert!(div.first_child("em").is_none());
    }

    #[rstest]
    #[case(r#"<p title=" Hi ">x</p>"#, Some(" Hi "))]
    #[case(r#"<p title="  ">x</p>"#, None)]
    #[case("<p>x</p>", None)]
    fn non_empty_attr_keeps_the_raw_value(#[case] html: &str, #[case] expected: Option<&str>) {
        let p = page_from_html(html);
        assert_eq!(
            p.by_tag("p").next().unwrap().non_empty_attr("title"),
            expected
        );
    }

    #[rstest]
    #[case("<p>text</p>", true)]
    #[case("<p> <b>bold</b> </p>", false)]
    #[case("<p></p>", false)]
    fn has_own_text(#[case] html: &str, #[case] expected: bool) {
        let p = page_from_html(html);
        let el = p.by_tag("p").next().unwrap();
        assert_eq!(el.has_own_text(), expected);
        assert!(!el.is_text());
    }

    #[rstest]
    // A structural path from body.
    #[case(
        "<body><main><div><p>a</p></div><div><button>x</button></div></main></body>",
        "button",
        0,
        "body > main > div:nth-of-type(2) > button"
    )]
    // Only same-tag siblings are counted.
    #[case(
        "<body><div><span>a</span><button>1</button><span>b</span><button>2</button></div></body>",
        "button",
        1,
        "body > div > button:nth-of-type(2)"
    )]
    // The path stops at the nearest unique id.
    #[case(
        r#"<body><div id="nav"><ul><li><a href="/">a</a></li></ul></div></body>"#,
        "a",
        0,
        "div#nav > ul > li > a"
    )]
    // The element's own unique id.
    #[case(
        r#"<body><div><button id="submit">x</button></div></body>"#,
        "button",
        0,
        "button#submit"
    )]
    // Duplicate and unusual ids are ignored.
    #[case(
        r#"<body><div id="d"><b id="a:b">x</b></div><div id="d"></div></body>"#,
        "b",
        0,
        "body > div:nth-of-type(1) > b"
    )]
    fn selector(#[case] html: &str, #[case] tag: &str, #[case] nth: usize, #[case] expected: &str) {
        let page = page_from_html(html);
        assert_eq!(page.by_tag(tag).nth(nth).unwrap().selector(), expected);
    }
}
