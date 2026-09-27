//! WAI-ARIA 1.2 rules, as listed in `standards/wai-aria-1.2-rules.md`.

mod attributes;
mod checks;
mod deprecated;
mod focus;
mod hidden;
mod idrefs;
mod keyshortcuts;
mod landmarks;
mod presentation;
mod reference;
mod roles;
mod sets;
mod structure;
mod support;
mod time_string;
mod tree;
mod usage;
mod values;
mod widgets;

pub use checks::rule_checks;
pub use reference::reference_url;
