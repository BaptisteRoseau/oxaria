//! Link discovery and URL identity for a site crawl: which hrefs on a
//! rendered page point somewhere worth scanning, and when two URLs count as
//! the same page.

use url::Url;

use crate::page::RenderedPage;

const LINK_TAGS: &[&str] = &["a", "area"];
const CRAWLABLE_SCHEMES: &[&str] = &["http", "https"];

pub fn extract_links(page: &RenderedPage, base: &Url) -> Vec<Url> {
    page.all()
        .filter(|el| LINK_TAGS.contains(&el.tag()))
        .filter_map(|el| el.attr("href"))
        .filter_map(|href| resolve(href, base))
        .collect()
}

fn resolve(href: &str, base: &Url) -> Option<Url> {
    let href = href.trim();
    if href.is_empty() || href.starts_with('#') {
        return None;
    }
    let mut url = base.join(href).ok()?;
    url.set_fragment(None);
    CRAWLABLE_SCHEMES.contains(&url.scheme()).then_some(url)
}

pub fn is_same_domain(url: &Url, origin: &Url) -> bool {
    url.host_str().is_some() && url.host_str() == origin.host_str()
}

/// A crawl never leaves one host, so the path alone identifies a page; query
/// strings and fragments are dropped so `/a?page=1` and `/a?page=2` don't
/// turn a paginated or faceted listing into an unbounded crawl.
pub fn visit_key(url: &Url) -> String {
    url.path().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn base() -> Url {
        Url::parse("https://example.com/docs/intro.html").unwrap()
    }

    fn paths(html: &str) -> Vec<String> {
        extract_links(&page_from_html(html), &base())
            .iter()
            .map(|url| url.to_string())
            .collect()
    }

    #[test]
    fn resolves_relative_root_relative_and_absolute_links() {
        let links = paths(
            r#"<a href="next.html">n</a><a href="/about">a</a>
               <a href="https://other.org/x">o</a>"#,
        );
        assert_eq!(
            links,
            [
                "https://example.com/docs/next.html",
                "https://example.com/about",
                "https://other.org/x",
            ]
        );
    }

    #[test]
    fn includes_image_map_areas() {
        let links = paths(r#"<map><area href="/region"></map>"#);
        assert_eq!(links, ["https://example.com/region"]);
    }

    #[test]
    fn skips_non_http_schemes_and_fragment_only_links() {
        let links = paths(
            r##"<a href="mailto:a@b.c">m</a><a href="tel:123">t</a>
               <a href="javascript:void(0)">j</a><a href="#top">f</a><a href="">e</a>"##,
        );
        assert!(links.is_empty(), "{links:?}");
    }

    #[test]
    fn strips_fragments() {
        let links = paths(r##"<a href="/faq#shipping">f</a>"##);
        assert_eq!(links, ["https://example.com/faq"]);
    }

    #[test]
    fn same_domain_compares_hosts() {
        let origin = base();
        assert!(is_same_domain(
            &Url::parse("http://example.com/other").unwrap(),
            &origin
        ));
        assert!(!is_same_domain(
            &Url::parse("https://blog.example.com/").unwrap(),
            &origin
        ));
        assert!(!is_same_domain(
            &Url::parse("https://other.org/").unwrap(),
            &origin
        ));
    }

    #[test]
    fn visit_key_ignores_query_and_fragment() {
        let a = Url::parse("https://example.com/list?page=1").unwrap();
        let b = Url::parse("https://example.com/list?page=2#top").unwrap();
        assert_eq!(visit_key(&a), "/list");
        assert_eq!(visit_key(&a), visit_key(&b));
    }

    #[test]
    fn visit_key_keeps_trailing_slash_distinct() {
        let a = Url::parse("https://example.com/a").unwrap();
        let b = Url::parse("https://example.com/a/").unwrap();
        assert_ne!(visit_key(&a), visit_key(&b));
    }
}
