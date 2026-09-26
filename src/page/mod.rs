//! Everything to do with turning the CLI's `path_or_url` argument into a
//! [`RenderedPage`] that every rule queries. See [`fetch::load_html`] for
//! how the raw HTML is obtained and [`render::render`] for how it's
//! converted, via the embedded litehtml engine, into a rendered snapshot.

mod fetch;
mod model;
mod render;
#[cfg(test)]
pub(crate) mod testutil;

#[cfg(test)]
pub use fetch::{ACCEPT_HTML, CHECKER_USER_AGENT};
pub use fetch::{client_builder, is_renderable, is_url, load_html};
pub use model::{ElementRef, RenderedPage};
pub use render::render;
