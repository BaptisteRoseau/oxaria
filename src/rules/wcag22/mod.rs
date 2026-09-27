//! WCAG 2.2 rules, as listed in `standards/wcag2.2-rules.md`.

mod aria;
mod authentication;
mod autocomplete;
mod checks;
mod contrast;
mod document;
mod focus;
mod forms;
mod frames;
mod headings;
mod images;
mod label_in_name;
mod language;
mod links;
mod multimedia;
mod navigation;
mod reference;
mod scripting;
mod tables;
mod target_size;
mod timing;

pub(crate) use autocomplete::is_valid_autocomplete;
pub use checks::rule_checks;
pub(crate) use contrast::{
    DEFAULT_CANVAS_BACKGROUND, Rgb, color_contrast, hex, is_large_text, passing_color,
    renders_own_text,
};
pub use reference::reference_url;
