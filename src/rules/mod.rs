//! Rule registry and orchestration. Every rule is a plain function taking the
//! rendered page (and, for threshold-based rules, the CLI-configured
//! [`CheckOptions`]), returning the [`Finding`]s it detected. [`check_html`]
//! renders a page and runs them in parallel.

mod act;
mod aria12;
mod aria_spec;
mod color;
mod css;
mod finding;
mod html_aria;
mod markup;
mod options;
mod registry;
mod spelling;
mod standard;
#[cfg(test)]
mod testutil;
mod wcag22;

pub use finding::{Finding, Severity};
pub use options::{
    CheckOptions, DEFAULT_CONTRAST_THRESHOLD, DEFAULT_LARGE_TEXT_CONTRAST_THRESHOLD,
    DEFAULT_TARGET_SIZE_THRESHOLD,
};
#[cfg(test)]
pub use registry::RuleCheck;
pub use registry::{Rule, check_html, rule_titles};
pub use standard::{Standard, StandardDef};
