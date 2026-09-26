//! Full-site scanning (`--full-site-scan`): crawl every same-host page
//! reachable from a start URL and check each one. See [`crawler::crawl`].

mod crawler;
mod links;
mod request_helper;

pub use crawler::crawl;
