//! ACT rules, as listed in `standards/act-rules.md`.

mod aria;
mod autocomplete;
mod checks;
mod contrast;
mod controls;
mod focus;
mod images;
mod language;
mod language_subtags;
mod name;
mod orientation;
mod reference;
mod refresh;
mod spacing;
mod tables;
mod zoom;

pub use checks::rule_checks;
pub use reference::reference_url;
