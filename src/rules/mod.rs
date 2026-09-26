//! Rule registry and orchestration. Every rule is a plain function taking the
//! rendered page and the CLI-configured [`CheckOptions`], returning the
//! [`Finding`]s it detected. [`run_all`] runs them in parallel.

mod act;
mod aria12;
mod finding;
mod html_aria;
mod options;
mod registry;
mod standard;
mod wcag22;

pub use finding::{Finding, Severity};
pub use options::{
    CheckOptions, DEFAULT_CONTRAST_THRESHOLD, DEFAULT_LARGE_TEXT_CONTRAST_THRESHOLD,
    DEFAULT_TARGET_SIZE_THRESHOLD,
};
pub use registry::{RuleCheck, run_all};
pub use standard::Standard;
