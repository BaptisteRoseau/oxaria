//! Converts raw HTML text into a [`RenderedPage`] using the embedded
//! litehtml rendering engine -- the single conversion path used for both
//! URL-fetched and file-read HTML.
//!
//! litehtml delegates font measurement to the embedder. Rather than shaping
//! real glyphs, this uses a plain average-character-width approximation:
//! none of wcag-checker's rules depend on pixel-accurate text wrapping, only
//! on computed color, computed font metrics, and the bounding boxes of
//! elements whose size is set by CSS (buttons, inputs, images, tables, ...)
//! rather than by an unconstrained run of inline text.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use litehtml::{
    Color, Document, DocumentContainer, DrawContext, Element, FontDescription, FontHandle,
    FontMetrics, MediaFeatures, MediaType, Position,
};

use crate::error::CheckerError;

use super::model::{Rect, RenderedElement, RenderedPage, TEXT_TAG};

const MASTER_CSS: &str = include_str!("master.css");
const VIEWPORT_WIDTH: f32 = 1280.0;
const VIEWPORT_HEIGHT: f32 = 1024.0;
const DEFAULT_FONT_SIZE: f32 = 16.0;
const DEFAULT_FONT_WEIGHT: u32 = 400;

const ATTRIBUTES_OF_INTEREST: &[&str] = &[
    "id",
    "href",
    "alt",
    "src",
    "type",
    "role",
    "required",
    "aria-required",
    "aria-label",
    "aria-labelledby",
    "aria-describedby",
    "aria-invalid",
    "title",
    "value",
    "for",
    "scope",
    "headers",
    "kind",
    "autoplay",
    "controls",
    "lang",
    "autocomplete",
];

pub fn render(html: &str) -> Result<RenderedPage, CheckerError> {
    // `document` holds an exclusive `&mut` borrow of `container` for its entire lifetime, so
    // `fonts` is a separate `Rc<RefCell<_>>` handle rather than a field read directly off
    // `container` -- that lets the tree walk below read font weights without needing to borrow
    // `container` itself, which is unavailable while `document` (and the `Element`s it hands
    // out) are still alive.
    let fonts = Rc::new(RefCell::new(HashMap::new()));
    let mut container = Container {
        next_font_id: 0,
        fonts: Rc::clone(&fonts),
    };
    let mut document = Document::from_html(html, &mut container, Some(MASTER_CSS), None)
        .map_err(|_| CheckerError::Render)?;
    let _ = document.render(VIEWPORT_WIDTH);

    let mut elements = Vec::new();
    if let Some(root) = document.root() {
        walk(root, None, &fonts.borrow(), &mut elements);
    }

    Ok(RenderedPage {
        elements,
        stylesheets: extract_stylesheets(html),
        url: None,
    })
}

#[derive(Clone, Copy)]
struct FontInfo {
    size: f32,
    weight: u32,
}

struct Container {
    next_font_id: usize,
    fonts: Rc<RefCell<HashMap<usize, FontInfo>>>,
}

impl DocumentContainer for Container {
    fn create_font(&mut self, descr: &FontDescription) -> (FontHandle, FontMetrics) {
        self.next_font_id += 1;
        let size = descr.size();
        let weight = descr.weight().max(0) as u32;
        self.fonts
            .borrow_mut()
            .insert(self.next_font_id, FontInfo { size, weight });

        let metrics = FontMetrics {
            font_size: size,
            height: size * 1.2,
            ascent: size,
            descent: size * 0.2,
            x_height: size * 0.5,
            ch_width: size * 0.5,
            draw_spaces: true,
            sub_shift: 0.0,
            super_shift: 0.0,
        };
        (FontHandle(self.next_font_id), metrics)
    }

    fn delete_font(&mut self, _font: FontHandle) {}

    fn text_width(&self, text: &str, font: FontHandle) -> f32 {
        let size = self
            .fonts
            .borrow()
            .get(&font.0)
            .map_or(DEFAULT_FONT_SIZE, |f| f.size);
        text.chars().count() as f32 * size * 0.5
    }

    fn draw_text(
        &mut self,
        _hdc: DrawContext,
        _text: &str,
        _font: FontHandle,
        _color: Color,
        _pos: Position,
    ) {
    }

    fn get_viewport(&self) -> Position {
        Position {
            x: 0.0,
            y: 0.0,
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
        }
    }

    fn get_media_features(&self) -> MediaFeatures {
        MediaFeatures {
            media_type: MediaType::Screen,
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            device_width: VIEWPORT_WIDTH,
            device_height: VIEWPORT_HEIGHT,
            color: 8,
            color_index: 0,
            monochrome: 0,
            resolution: 96.0,
        }
    }
}

fn walk(
    node: Element,
    parent: Option<usize>,
    fonts: &HashMap<usize, FontInfo>,
    elements: &mut Vec<RenderedElement>,
) -> usize {
    let index = elements.len();
    elements.push(reserved_slot(parent));

    elements[index] = match node.is_text() {
        true => text_element(&node, parent),
        false => rendered_element(&node, parent, fonts),
    };

    let mut children = Vec::with_capacity(node.children_count());
    for i in 0..node.children_count() {
        if let Some(child) = node.child_at(i) {
            children.push(walk(child, Some(index), fonts, elements));
        }
    }
    elements[index].children = children;

    index
}

fn reserved_slot(parent: Option<usize>) -> RenderedElement {
    RenderedElement {
        tag: String::new(),
        attrs: Vec::new(),
        own_text: String::new(),
        parent,
        children: Vec::new(),
        color: (0, 0, 0),
        background_color: None,
        font_size_px: DEFAULT_FONT_SIZE,
        font_weight: DEFAULT_FONT_WEIGHT,
        bounding_box: Rect::default(),
    }
}

fn text_element(node: &Element, parent: Option<usize>) -> RenderedElement {
    RenderedElement {
        tag: TEXT_TAG.to_string(),
        own_text: node.get_text(),
        ..reserved_slot(parent)
    }
}

fn rendered_element(
    node: &Element,
    parent: Option<usize>,
    fonts: &HashMap<usize, FontInfo>,
) -> RenderedElement {
    let color = to_rgb(node.color());
    let font = fonts.get(&node.font().0);
    let placement = node.placement();

    RenderedElement {
        tag: node.tag_name(),
        attrs: attributes(node),
        color,
        background_color: effective_background_color(*node),
        font_size_px: font.map_or(node.font_size(), |f| f.size),
        font_weight: font.map_or(DEFAULT_FONT_WEIGHT, |f| f.weight),
        bounding_box: Rect {
            x: placement.x,
            y: placement.y,
            width: placement.width,
            height: placement.height,
        },
        ..reserved_slot(parent)
    }
}

fn attributes(node: &Element) -> Vec<(String, String)> {
    ATTRIBUTES_OF_INTEREST
        .iter()
        .filter_map(|name| node.attr(name).map(|value| (name.to_string(), value)))
        .collect()
}

/// Walks up through ancestors (litehtml's own tree, not our arena, since
/// this runs before the arena exists) for the nearest non-transparent
/// declared `background-color`. CSS backgrounds never inherit automatically,
/// but a page's *effective* visual background comes from whichever ancestor
/// box is actually painted -- exactly what a real browser shows.
fn effective_background_color(mut node: Element) -> Option<(u8, u8, u8)> {
    loop {
        if let Some(color) = node.background_color() {
            return Some(to_rgb(color));
        }
        node = node.parent()?;
    }
}

fn to_rgb(color: Color) -> (u8, u8, u8) {
    (color.r, color.g, color.b)
}

pub(crate) fn extract_stylesheets(html: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let mut stylesheets = Vec::new();
    let mut search_from = 0usize;
    while let Some(open_start) = lower[search_from..].find("<style") {
        let open_start = search_from + open_start;
        let Some(open_end) = lower[open_start..].find('>') else {
            break;
        };
        let content_start = open_start + open_end + 1;
        let Some(close_start) = lower[content_start..].find("</style") else {
            break;
        };
        let content_end = content_start + close_start;
        stylesheets.push(html[content_start..content_end].to_string());
        search_from = content_end;
    }
    stylesheets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_tag_and_attributes() {
        let page = render("<img id=\"pic\" src=\"a.jpg\" alt=\"A red bicycle\">").unwrap();
        let img = page.by_tag("img").next().unwrap();
        assert_eq!(img.attr("id"), Some("pic"));
        assert_eq!(img.attr("src"), Some("a.jpg"));
        assert_eq!(img.attr("alt"), Some("A red bicycle"));
    }

    #[test]
    fn renders_computed_color_from_class_selector() {
        let html = "<html><head><style>.low{color:#999999}</style></head>\
                     <body><p class=\"low\">Text</p></body></html>";
        let page = render(html).unwrap();
        let p = page.by_tag("p").next().unwrap();
        assert_eq!(p.color(), (0x99, 0x99, 0x99));
    }

    #[test]
    fn renders_own_background_color() {
        let html = "<p style=\"background-color:#ffffff\">Text</p>";
        let page = render(html).unwrap();
        let p = page.by_tag("p").next().unwrap();
        assert_eq!(p.background_color(), Some((0xff, 0xff, 0xff)));
    }

    #[test]
    fn background_color_walks_up_to_ancestor() {
        let html = "<body style=\"background-color:#eeeeee\"><span>Text</span></body>";
        let page = render(html).unwrap();
        let span = page.by_tag("span").next().unwrap();
        assert_eq!(span.background_color(), Some((0xee, 0xee, 0xee)));
    }

    #[test]
    fn renders_bounding_box() {
        let html = "<button style=\"width:40px;height:24px\">Go</button>";
        let page = render(html).unwrap();
        let button = page.by_tag("button").next().unwrap();
        let bbox = button.bounding_box();
        assert!((bbox.width - 40.0).abs() < 1.0);
        assert!((bbox.height - 24.0).abs() < 1.0);
    }

    #[test]
    fn text_content_is_collected() {
        let page = render("<button>Submit</button>").unwrap();
        let button = page.by_tag("button").next().unwrap();
        assert_eq!(button.text(), "Submit");
    }

    #[test]
    fn extract_stylesheets_finds_style_block() {
        let html = "<head><style>a:focus{outline:none}</style></head>";
        let sheets = extract_stylesheets(html);
        assert_eq!(sheets, vec!["a:focus{outline:none}".to_string()]);
    }

    #[test]
    fn extract_stylesheets_handles_multiple_blocks() {
        let html = "<style>a{color:red}</style><style>b{color:blue}</style>";
        let sheets = extract_stylesheets(html);
        assert_eq!(
            sheets,
            vec!["a{color:red}".to_string(), "b{color:blue}".to_string()]
        );
    }

    #[test]
    fn extract_stylesheets_is_not_shifted_by_non_ascii_text() {
        let html = "<p>İstanbul İzmir</p><STYLE>a:focus{outline:none}</STYLE>";
        let sheets = extract_stylesheets(html);
        assert_eq!(sheets, vec!["a:focus{outline:none}".to_string()]);
    }

    #[test]
    fn extract_stylesheets_empty_when_none_present() {
        assert!(extract_stylesheets("<p>No styles here</p>").is_empty());
    }

    #[test]
    fn invalid_html_with_null_byte_is_a_render_error() {
        let result = render("<p>\0</p>");
        assert!(result.is_err());
    }
}
