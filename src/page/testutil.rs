//! Test-only conversion from plain HTML text into a [`RenderedPage`], via
//! `scraper`/`html5ever` (a dev-dependency only -- production code renders
//! exclusively through the embedded litehtml engine, see [`super::render`]).
//!
//! Structure, tags, attributes, and text come straight from parsing.
//! `color`/`background-color`/`font-size`/`font-weight`/`width`/`height`
//! come from a simple, non-cascading read of each element's own inline
//! `style` attribute (no class/cascade resolution), for the few rules that
//! need to exercise contrast- or target-size-sensitive logic.

use scraper::{ElementRef, Html, Node};

use super::model::{Rect, RenderedElement, RenderedPage, TEXT_TAG};
use super::render::extract_stylesheets;

const DEFAULT_COLOR: (u8, u8, u8) = (0, 0, 0);
const DEFAULT_FONT_SIZE_PX: f32 = 16.0;
const DEFAULT_FONT_WEIGHT: u32 = 400;

pub fn page_from_html(html: &str) -> RenderedPage {
    let document = Html::parse_document(html);
    let mut elements = Vec::new();
    walk(document.root_element(), None, &mut elements);

    RenderedPage {
        elements,
        stylesheets: extract_stylesheets(html),
    }
}

fn walk(element: ElementRef, parent: Option<usize>, elements: &mut Vec<RenderedElement>) -> usize {
    let index = elements.len();
    let style = element.value().attr("style");

    elements.push(RenderedElement {
        tag: element.value().name().to_string(),
        attrs: element
            .value()
            .attrs()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect(),
        own_text: String::new(),
        parent,
        children: Vec::new(),
        color: style.and_then(declared_color).unwrap_or(DEFAULT_COLOR),
        background_color: style.and_then(declared_background_color),
        font_size_px: style
            .and_then(declared_font_size)
            .unwrap_or(DEFAULT_FONT_SIZE_PX),
        font_weight: style
            .and_then(declared_font_weight)
            .unwrap_or(DEFAULT_FONT_WEIGHT),
        bounding_box: style.and_then(declared_bounding_box).unwrap_or_default(),
    });

    let mut children = Vec::new();
    for child in element.children() {
        match child.value() {
            Node::Element(_) => {
                let child_ref = ElementRef::wrap(child).expect("Node::Element wraps as ElementRef");
                children.push(walk(child_ref, Some(index), elements));
            }
            Node::Text(text) => {
                children.push(push_text(&text.text, index, elements));
            }
            _ => {}
        }
    }
    elements[index].children = children;

    index
}

fn push_text(text: &str, parent: usize, elements: &mut Vec<RenderedElement>) -> usize {
    let index = elements.len();
    elements.push(RenderedElement {
        tag: TEXT_TAG.to_string(),
        attrs: Vec::new(),
        own_text: text.to_string(),
        parent: Some(parent),
        children: Vec::new(),
        color: DEFAULT_COLOR,
        background_color: None,
        font_size_px: DEFAULT_FONT_SIZE_PX,
        font_weight: DEFAULT_FONT_WEIGHT,
        bounding_box: Rect::default(),
    });
    index
}

fn style_property<'a>(style: &'a str, property: &str) -> Option<&'a str> {
    style.split(';').find_map(|declaration| {
        let (name, value) = declaration.split_once(':')?;
        name.trim()
            .eq_ignore_ascii_case(property)
            .then(|| value.trim())
    })
}

fn declared_color(style: &str) -> Option<(u8, u8, u8)> {
    parse_color(style_property(style, "color")?)
}

fn declared_background_color(style: &str) -> Option<(u8, u8, u8)> {
    parse_color(style_property(style, "background-color")?)
}

fn declared_font_size(style: &str) -> Option<f32> {
    pixel_value(style, "font-size")
}

fn declared_font_weight(style: &str) -> Option<u32> {
    let value = style_property(style, "font-weight")?;
    match value.parse::<u32>() {
        Ok(weight) => Some(weight),
        Err(_) if value.eq_ignore_ascii_case("bold") => Some(700),
        Err(_) => None,
    }
}

fn declared_bounding_box(style: &str) -> Option<Rect> {
    Some(Rect {
        x: 0.0,
        y: 0.0,
        width: pixel_value(style, "width")?,
        height: pixel_value(style, "height")?,
    })
}

fn pixel_value(style: &str, property: &str) -> Option<f32> {
    style_property(style, property)?
        .trim_end_matches("px")
        .trim()
        .parse()
        .ok()
}

fn parse_color(value: &str) -> Option<(u8, u8, u8)> {
    let value = value.trim();
    match () {
        _ if value.starts_with('#') => parse_hex_color(value),
        _ if value.starts_with("rgb") => parse_rgb_function(value),
        _ => named_color(value),
    }
}

fn parse_hex_color(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.trim_start_matches('#');
    match hex.len() {
        3 => {
            let expand = |c: char| u8::from_str_radix(&c.to_string().repeat(2), 16).ok();
            let mut chars = hex.chars();
            Some((
                expand(chars.next()?)?,
                expand(chars.next()?)?,
                expand(chars.next()?)?,
            ))
        }
        6 => Some((
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        )),
        _ => None,
    }
}

fn parse_rgb_function(value: &str) -> Option<(u8, u8, u8)> {
    let inner = value.split_once('(')?.1.trim_end_matches(')');
    let mut parts = inner.split(',').map(|part| part.trim().parse::<u8>());
    Some((
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
    ))
}

fn named_color(name: &str) -> Option<(u8, u8, u8)> {
    Some(match name.to_lowercase().as_str() {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 128, 0),
        "blue" => (0, 0, 255),
        "gray" | "grey" => (128, 128, 128),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tag_and_attributes() {
        let page = page_from_html(r#"<img id="pic" src="a.jpg" alt="A red bicycle">"#);
        let img = page.by_tag("img").next().unwrap();
        assert_eq!(img.attr("id"), Some("pic"));
        assert_eq!(img.attr("src"), Some("a.jpg"));
        assert_eq!(img.attr("alt"), Some("A red bicycle"));
    }

    #[test]
    fn parses_nested_text() {
        let page = page_from_html("<button>Submit</button>");
        let button = page.by_tag("button").next().unwrap();
        assert_eq!(button.text(), "Submit");
    }

    #[test]
    fn parses_inline_style_color_and_background() {
        let page = page_from_html(r#"<p style="color:#999999;background-color:#ffffff">Text</p>"#);
        let p = page.by_tag("p").next().unwrap();
        assert_eq!(p.color(), (0x99, 0x99, 0x99));
        assert_eq!(p.background_color(), Some((0xff, 0xff, 0xff)));
    }

    #[test]
    fn parses_inline_style_bounding_box() {
        let page = page_from_html(r#"<button style="width:40px;height:24px">Go</button>"#);
        let button = page.by_tag("button").next().unwrap();
        let bbox = button.bounding_box();
        assert_eq!(bbox.width, 40.0);
        assert_eq!(bbox.height, 24.0);
    }

    #[test]
    fn defaults_when_no_style_declared() {
        let page = page_from_html("<p>Text</p>");
        let p = page.by_tag("p").next().unwrap();
        assert_eq!(p.color(), DEFAULT_COLOR);
        assert_eq!(p.background_color(), None);
        assert_eq!(p.font_size_px(), DEFAULT_FONT_SIZE_PX);
    }

    #[test]
    fn extracts_style_tag_text() {
        let page = page_from_html("<head><style>a:focus{outline:none}</style></head>");
        assert_eq!(page.stylesheet_text(), "a:focus{outline:none}");
    }
}
