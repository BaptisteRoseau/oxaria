//! WCAG 2.4.4 / 2.4.9 checks for link text.

use std::collections::BTreeMap;

use url::Url;

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

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

/// H30 (F89 for an image-only link): a link's accessible name must not be empty, and must not be
/// one of the generic phrases ("click here", "more", …) that reads as meaningless out of context.
pub fn check_non_descriptive_link_text(page: &RenderedPage) -> Vec<Finding> {
    page.all()
        .filter(|el| is_link(*el))
        .filter_map(non_descriptive_finding)
        .collect()
}

fn non_descriptive_finding(link: ElementRef) -> Option<Finding> {
    let name = link.accessible_name();
    let href = href_of(link);
    if name.trim().is_empty() {
        return Some(empty_link_finding(link, href));
    }
    is_generic_phrase(&name).then(|| {
        Finding::error(
            "H30",
            format!("link to \"{href}\" uses non-descriptive text \"{name}\""),
        )
        .at(link)
        .help(format!(
            "replace \"{}\" with text saying where the link goes, so it makes sense on its own \
             (screen readers list a page's links out of context)",
            name.trim()
        ))
    })
}

/// F89 is the failure specific to a link whose only content is an image with no text
/// alternative; any other nameless link is reported under H30.
fn empty_link_finding(link: ElementRef, href: &str) -> Finding {
    match link.descendants().any(|el| el.tag() == "img") {
        true => Finding::error(
            "F89",
            format!("link to \"{href}\" contains only an image with no text alternative"),
        )
        .at(link)
        .help("give the image an alt saying where the link goes, e.g. alt=\"Acme home page\""),
        false => Finding::error("H30", format!("link to \"{href}\" has no link text"))
            .at(link)
            .help(
                "add text saying where the link goes, or an aria-label=\"...\" if it only shows an icon",
            ),
    }
}

fn is_generic_phrase(name: &str) -> bool {
    GENERIC_LINK_PHRASES.contains(&name.trim().to_lowercase().as_str())
}

fn href_of(link: ElementRef<'_>) -> &str {
    link.attr("href").unwrap_or("")
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

/// LNK001: the same link text pointing to different destinations is ambiguous when users navigate a
/// page's links out of context (e.g. a screen reader's links list).
pub fn check_ambiguous_duplicate_link_text(page: &RenderedPage) -> Vec<Finding> {
    group_links_by_text(page)
        .into_iter()
        .filter_map(|(text, hrefs)| duplicate_text_finding(&text, hrefs))
        .collect()
}

fn group_links_by_text(page: &RenderedPage) -> BTreeMap<String, Vec<String>> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for link in page.all().filter(|el| is_link(*el)) {
        let name = link.accessible_name().trim().to_lowercase();
        if name.len() < 3 {
            continue;
        }
        groups
            .entry(name)
            .or_default()
            .push(destination(href_of(link), page.url.as_ref()));
    }
    groups
}

fn duplicate_text_finding(text: &str, mut distinct_hrefs: Vec<String>) -> Option<Finding> {
    distinct_hrefs.sort();
    distinct_hrefs.dedup();
    (distinct_hrefs.len() > 1).then(|| {
        Finding::error(
            "LNK001",
            format!(
                "link text \"{text}\" is used for {} different destinations: {}",
                distinct_hrefs.len(),
                distinct_hrefs.join(", ")
            ),
        )
        .help(format!(
            "make each link's text unique, e.g. \"{text} about <topic>\", \
             or link them all to the same URL if they are the same page"
        ))
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::{page_from_html, page_from_html_at};
    use crate::rules::testutil::{help, rule_ids, run};

    #[test]
    fn empty_image_link_help_points_at_the_images_alt() {
        let html = r#"<a href="/"><img src="logo.png"></a><a href="/x"></a>"#;
        let findings = run(check_non_descriptive_link_text, html);
        let help = |index: usize| findings[index].help.as_deref().unwrap();
        assert!(help(0).starts_with("give the image an alt"), "{}", help(0));
        assert!(help(1).starts_with("add text"), "{}", help(1));
    }

    #[test]
    fn generic_link_help_quotes_the_text() {
        let html = r#"<a href="/report.pdf">Click here</a>"#;
        assert!(
            help(check_non_descriptive_link_text, html).starts_with(r#"replace "Click here" with"#)
        );
    }

    #[rstest]
    #[case(
        r#"<a href="scores.html"><img src="football.gif" alt=""></a>
           <a href="scores.html">Football Scoreboard</a>"#,
        &["F89"]
    )]
    #[case(r#"<a href="/report.pdf"></a>"#, &["H30"])]
    #[case(r#"<a href="/report.pdf">Click here</a>"#, &["H30"])]
    #[case(r#"<a href="/report.pdf">Download the 2026 Annual Report</a>"#, &[])]
    // aria-label overrides the generic visible text.
    #[case(r#"<a href="/report.pdf" aria-label="Download the 2026 report">Click here</a>"#, &[])]
    fn non_descriptive_link_text(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(
            rule_ids(check_non_descriptive_link_text, html),
            expected,
            "{html}"
        );
    }

    #[test]
    fn ambiguous_link_texts_are_reported_in_a_stable_order() {
        let html = r#"<a href="/1">Pricing</a><a href="/2">Pricing</a><a href="/3">About us</a><a href="/4">About us</a>"#;
        let findings = run(check_ambiguous_duplicate_link_text, html);
        assert!(findings[0].message.contains("\"about us\""), "{findings:?}");
        assert!(findings[1].message.contains("\"pricing\""), "{findings:?}");
    }

    #[rstest]
    #[case(
        r#"<a href="/products/1">Read more</a><a href="/products/2">Read more</a>"#,
        None,
        1
    )]
    #[case(
        r#"<a href="/products/1">Wireless Headphones</a><a href="/products/1">Wireless Headphones</a>"#,
        None,
        0
    )]
    // github.com: "pricing" linked as both `/pricing` and `https://github.com/pricing`.
    #[case(
        r#"<a href="/pricing">Pricing</a><a href="https://github.com/pricing">Pricing</a>"#,
        Some("https://github.com/"),
        0
    )]
    // Page-relative and root-relative hrefs are compared resolved.
    #[case(
        r#"<a href="guide">User guide</a><a href="/docs/guide">User guide</a>"#,
        Some("https://example.com/docs/intro"),
        0
    )]
    // nodejs.org API docs: "index" linked as `index.html` and `index.html#index`.
    #[case(
        r##"<a href="index.html">Index page</a><a href="index.html#index">Index page</a>"##,
        None,
        0
    )]
    // ubuntu.com: the same page linked with and without a `?_gl=` tracking parameter.
    #[case(
        r#"<a href="/managed-infrastructure">Managed infrastructure</a>
           <a href="https://ubuntu.com/managed-infrastructure?_gl=1*e5c2b5">Managed infrastructure</a>"#,
        Some("https://ubuntu.com/"),
        0
    )]
    // code.visualstudio.com: "Download" linked as both `/Download` and `/download`.
    #[case(
        r#"<a href="/Download">Download</a><a href="/download">Download</a>"#,
        None,
        0
    )]
    // pypi.org: "Python Software Foundation" linked to `/psf-landing` and `/psf-landing/`.
    #[case(
        r#"<a href="https://www.python.org/psf-landing">Python Software Foundation</a>
           <a href="https://www.python.org/psf-landing/">Python Software Foundation</a>"#,
        Some("https://pypi.org/"),
        0
    )]
    // ubuntu.com: Multipass linked as both `http://` and `https://canonical.com/multipass`.
    #[case(
        r#"<a href="http://canonical.com/multipass">Multipass VMs</a>
           <a href="https://canonical.com/multipass">Multipass VMs</a>"#,
        Some("https://ubuntu.com/navigation"),
        0
    )]
    // Different paths are still flagged with a page URL.
    #[case(
        r#"<a href="/products/1">Read more</a><a href="https://shop.example/products/2">Read more</a>"#,
        Some("https://shop.example/"),
        1
    )]
    // Text shorter than 3 characters is ignored.
    #[case(r#"<a href="/a">Go</a><a href="/b">Go</a>"#, None, 0)]
    fn ambiguous_duplicate_link_text(
        #[case] html: &str,
        #[case] url: Option<&str>,
        #[case] expected: usize,
    ) {
        let page = match url {
            Some(url) => page_from_html_at(html, url),
            None => page_from_html(html),
        };
        let findings = check_ambiguous_duplicate_link_text(&page);
        assert_eq!(findings.len(), expected, "{html}");
        assert!(findings.iter().all(|finding| finding.rule_id == "LNK001"));
    }

    #[test]
    fn root_and_empty_href_keep_their_distinct_destinations() {
        assert_eq!(comparable("/"), "/");
        assert_eq!(comparable(""), "");
    }

    #[test]
    fn same_path_on_another_host_is_still_a_different_destination() {
        let p = page_from_html_at(
            r#"<a href="/pricing">Pricing</a><a href="https://other.example/pricing">Pricing</a>"#,
            "https://github.com/",
        );
        let findings = check_ambiguous_duplicate_link_text(&p);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0]
                .message
                .contains("//other.example/pricing, /pricing"),
            "{}",
            findings[0].message
        );
    }
}
