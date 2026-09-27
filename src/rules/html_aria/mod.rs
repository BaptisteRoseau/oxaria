//! ARIA in HTML rules, as listed in `standards/html-aria-rules.md`.

mod attributes;
mod checks;
mod content_model;
mod deprecated;
mod markup;
mod native_attributes;
mod reference;
mod roles;
mod syntax;

pub use checks::{RULE_IDS, rule_checks};
pub use reference::reference_url;
