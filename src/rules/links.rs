//! WCAG 2.4.4 / 2.4.9 checks for link text.

use std::collections::BTreeMap;

use url::Url;

use crate::page::{self, ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

const GENERIC_LINK_PHRASES: &[&str] = &[
    "click here",
    "here",
    "more",
    "read more",
    "learn more",
    "link",
    "this link",
    "click",
];

fn is_link(el: ElementRef) -> bool {
    el.tag() == "a" && el.has_attr("href")
}

/// H30: a link's accessible name must not be empty, and must not be one of the generic phrases
/// ("click here", "more", …) that reads as meaningless out of context.
pub fn check_non_descriptive_link_text(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.select(is_link)
        .into_iter()
        .filter_map(|link| non_descriptive_finding(page, link))
        .collect()
}

fn non_descriptive_finding(page: &RenderedPage, link: ElementRef) -> Option<Finding> {
    let name = page::accessible_name(page, link);
    let href = href_of(link);
    if name.trim().is_empty() {
        return Some(
            Finding::error("H30", format!("link to \"{href}\" has no link text")).at(link),
        );
    }
    is_generic_phrase(&name).then(|| {
        Finding::error(
            "H30",
            format!("link to \"{href}\" uses non-descriptive text \"{name}\""),
        )
        .at(link)
    })
}

fn is_generic_phrase(name: &str) -> bool {
    GENERIC_LINK_PHRASES.contains(&name.trim().to_lowercase().as_str())
}

fn href_of(link: ElementRef) -> String {
    link.attr("href").unwrap_or("").to_string()
}

/// Reduces an href to the page it actually leads to, so hrefs that are only
/// spelled differently don't count as different destinations: the fragment
/// (`index.html` vs `index.html#index`) and query string (tracking params
/// like `?_gl=...`) are dropped, and when the page's own URL is known, a
/// same-host absolute link is reduced to its path (`https://github.com/pricing`
/// vs `/pricing`). Links to other hosts keep their host but lose their
/// scheme (`http://canonical.com/x` vs `https://canonical.com/x`).
///
/// Case and a trailing slash are ignored too (`/Download` vs `/download`,
/// `psf-landing` vs `psf-landing/`): strictly these are different URLs, but
/// sites serve them as one page, and to a user reading identical link text
/// they are the same destination. This is only for comparing link text --
/// the crawler's visit key keeps them distinct.
fn destination(href: &str, page_url: Option<&Url>) -> String {
    let href = href.split(['#', '?']).next().unwrap_or_default();
    let resolved = match page_url.map(|base| (base, base.join(href))) {
        Some((base, Ok(url))) if url.host_str() == base.host_str() => url.path().to_string(),
        Some((_, Ok(url))) => without_scheme(&url),
        _ => href.to_string(),
    };
    comparable(&resolved)
}

fn without_scheme(url: &Url) -> String {
    let port = url
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    format!(
        "//{}{port}{}",
        url.host_str().unwrap_or_default(),
        url.path()
    )
}

fn comparable(destination: &str) -> String {
    let lowercase = destination.to_lowercase();
    match lowercase.trim_end_matches('/') {
        "" => lowercase,
        trimmed => trimmed.to_string(),
    }
}

/// F84: the same link text pointing to different destinations is ambiguous when users navigate a
/// page's links out of context (e.g. a screen reader's links list).
pub fn check_ambiguous_duplicate_link_text(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    group_links_by_text(page)
        .into_iter()
        .filter_map(|(text, hrefs)| duplicate_text_finding(&text, hrefs))
        .collect()
}

fn group_links_by_text(page: &RenderedPage) -> BTreeMap<String, Vec<String>> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for link in page.select(is_link) {
        let name = page::accessible_name(page, link).trim().to_lowercase();
        if name.len() < 3 {
            continue;
        }
        groups
            .entry(name)
            .or_default()
            .push(destination(&href_of(link), page.url.as_ref()));
    }
    groups
}

fn duplicate_text_finding(text: &str, hrefs: Vec<String>) -> Option<Finding> {
    let mut distinct_hrefs: Vec<String> = hrefs;
    distinct_hrefs.sort();
    distinct_hrefs.dedup();
    (distinct_hrefs.len() > 1).then(|| {
        Finding::error(
            "F84",
            format!(
                "link text \"{text}\" is used for {} different destinations: {}",
                distinct_hrefs.len(),
                distinct_hrefs.join(", ")
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn options() -> CheckOptions {
        CheckOptions {
            contrast_threshold: 4.5,
            large_text_contrast_threshold: 3.0,
            target_size_threshold: 24.0,
        }
    }

    #[test]
    fn empty_link_text_is_flagged() {
        let p = page_from_html(r#"<a href="/report.pdf"></a>"#);
        let findings = check_non_descriptive_link_text(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H30");
    }

    #[test]
    fn click_here_is_flagged() {
        let p = page_from_html(r#"<a href="/report.pdf">Click here</a>"#);
        assert_eq!(check_non_descriptive_link_text(&p, &options()).len(), 1);
    }

    #[test]
    fn descriptive_link_text_is_not_flagged() {
        let p = page_from_html(r#"<a href="/report.pdf">Download the 2026 Annual Report</a>"#);
        assert!(check_non_descriptive_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn aria_label_overrides_generic_visible_text() {
        let p = page_from_html(
            r#"<a href="/report.pdf" aria-label="Download the 2026 report">Click here</a>"#,
        );
        assert!(check_non_descriptive_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn same_text_different_hrefs_is_flagged() {
        let p = page_from_html(
            r#"<a href="/products/1">Read more</a><a href="/products/2">Read more</a>"#,
        );
        let findings = check_ambiguous_duplicate_link_text(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F84");
    }

    #[test]
    fn ambiguous_link_texts_are_reported_in_a_stable_order() {
        let p = page_from_html(
            r#"<a href="/1">Pricing</a><a href="/2">Pricing</a><a href="/3">About us</a><a href="/4">About us</a>"#,
        );
        let findings = check_ambiguous_duplicate_link_text(&p, &options());
        assert!(findings[0].message.contains("\"about us\""), "{findings:?}");
        assert!(findings[1].message.contains("\"pricing\""), "{findings:?}");
    }

    #[test]
    fn same_text_same_href_is_not_flagged() {
        let p = page_from_html(
            r#"<a href="/products/1">Wireless Headphones</a><a href="/products/1">Wireless Headphones</a>"#,
        );
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }

    fn page_at(html: &str, url: &str) -> RenderedPage {
        RenderedPage {
            url: Some(Url::parse(url).unwrap()),
            ..page_from_html(html)
        }
    }

    #[test]
    fn relative_and_same_host_absolute_hrefs_are_the_same_destination() {
        // github.com: "pricing" linked as both `/pricing` and `https://github.com/pricing`.
        let p = page_at(
            r#"<a href="/pricing">Pricing</a><a href="https://github.com/pricing">Pricing</a>"#,
            "https://github.com/",
        );
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn page_relative_and_root_relative_hrefs_are_compared_resolved() {
        let p = page_at(
            r#"<a href="guide">User guide</a><a href="/docs/guide">User guide</a>"#,
            "https://example.com/docs/intro",
        );
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn hrefs_differing_only_by_fragment_are_the_same_destination() {
        // nodejs.org API docs: "index" linked as `index.html` and `index.html#index`.
        let p = page_from_html(
            r##"<a href="index.html">Index page</a><a href="index.html#index">Index page</a>"##,
        );
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn hrefs_differing_only_by_query_string_are_the_same_destination() {
        // ubuntu.com: the same page linked with and without a `?_gl=` tracking parameter.
        let p = page_at(
            r#"<a href="/managed-infrastructure">Managed infrastructure</a>
               <a href="https://ubuntu.com/managed-infrastructure?_gl=1*e5c2b5">Managed infrastructure</a>"#,
            "https://ubuntu.com/",
        );
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn hrefs_differing_only_by_case_are_the_same_destination() {
        // code.visualstudio.com: "Download" linked as both `/Download` and `/download`.
        let p =
            page_from_html(r#"<a href="/Download">Download</a><a href="/download">Download</a>"#);
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn hrefs_differing_only_by_trailing_slash_are_the_same_destination() {
        // pypi.org: "Python Software Foundation" linked to `/psf-landing` and `/psf-landing/`.
        let p = page_at(
            r#"<a href="https://www.python.org/psf-landing">Python Software Foundation</a>
               <a href="https://www.python.org/psf-landing/">Python Software Foundation</a>"#,
            "https://pypi.org/",
        );
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn http_and_https_links_to_the_same_page_are_the_same_destination() {
        // ubuntu.com: Multipass linked as both `http://` and `https://canonical.com/multipass`.
        let p = page_at(
            r#"<a href="http://canonical.com/multipass">Multipass VMs</a>
               <a href="https://canonical.com/multipass">Multipass VMs</a>"#,
            "https://ubuntu.com/navigation",
        );
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }

    #[test]
    fn root_and_empty_href_keep_their_distinct_destinations() {
        assert_eq!(comparable("/"), "/");
        assert_eq!(comparable(""), "");
    }

    #[test]
    fn same_path_on_another_host_is_still_a_different_destination() {
        let p = page_at(
            r#"<a href="/pricing">Pricing</a><a href="https://other.example/pricing">Pricing</a>"#,
            "https://github.com/",
        );
        let findings = check_ambiguous_duplicate_link_text(&p, &options());
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0]
                .message
                .contains("//other.example/pricing, /pricing"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn different_paths_are_still_flagged_with_a_page_url() {
        let p = page_at(
            r#"<a href="/products/1">Read more</a><a href="https://shop.example/products/2">Read more</a>"#,
            "https://shop.example/",
        );
        assert_eq!(check_ambiguous_duplicate_link_text(&p, &options()).len(), 1);
    }

    #[test]
    fn short_text_is_ignored_for_duplicate_check() {
        let p = page_from_html(r#"<a href="/a">Go</a><a href="/b">Go</a>"#);
        assert!(check_ambiguous_duplicate_link_text(&p, &options()).is_empty());
    }
}
