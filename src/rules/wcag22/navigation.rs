//! WCAG 2.4.1 check for a "skip to main content" link.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

/// G1: a page should offer a link (usually the first focusable element) that jumps past repeated
/// navigation straight to the main content, and its target must actually exist.
pub fn check_missing_skip_link(page: &RenderedPage) -> Vec<Finding> {
    match find_skip_link(page) {
        None => vec![
            Finding::error(
                "G1",
                "no skip link found (an <a href=\"#...\"> whose text mentions \"skip\")"
                    .to_string(),
            )
            .help(
                "make the first link in <body> <a href=\"#main\">Skip to main content</a>, \
             and mark the main content <main id=\"main\">",
            ),
        ],
        Some(link) if !target_exists(page, link) => {
            vec![
                Finding::error("G1", dangling_skip_link_message(link))
                    .at(link)
                    .help(dangling_skip_link_help(link)),
            ]
        }
        Some(_) => Vec::new(),
    }
}

fn find_skip_link(page: &RenderedPage) -> Option<ElementRef<'_>> {
    page.all()
        .filter(|el| is_fragment_link(*el))
        .find(|link| link.text().to_lowercase().contains("skip"))
}

fn is_fragment_link(el: ElementRef) -> bool {
    el.tag() == "a" && el.attr("href").is_some_and(|href| href.starts_with('#'))
}

fn target_exists(page: &RenderedPage, link: ElementRef) -> bool {
    fragment_id(link).is_some_and(|id| page.element_by_id(id).is_some())
}

fn fragment_id(link: ElementRef<'_>) -> Option<&str> {
    link.attr("href")?.strip_prefix('#')
}

fn dangling_skip_link_help(link: ElementRef) -> String {
    let id = fragment_id(link).unwrap_or("main");
    format!("add id=\"{id}\" to the main content, e.g. <main id=\"{id}\">")
}

fn dangling_skip_link_message(link: ElementRef) -> String {
    let href = link.attr("href").unwrap_or("#");
    format!("skip link target \"{href}\" does not exist in the document")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::{help, rule_ids};

    #[test]
    fn dangling_skip_link_help_names_the_missing_id() {
        let html = r##"<a href="#content">Skip to content</a><main></main>"##;
        assert_eq!(
            help(check_missing_skip_link, html),
            r#"add id="content" to the main content, e.g. <main id="content">"#
        );
    }

    #[rstest]
    #[case(r#"<body><nav></nav><main id="main"></main></body>"#, &["G1"])]
    #[case(
        r##"<body><a href="#main">Skip to main content</a><main id="main"></main></body>"##,
        &[]
    )]
    // The skip link's target is missing.
    #[case(r##"<body><a href="#main">Skip to main content</a></body>"##, &["G1"])]
    fn missing_skip_link(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_missing_skip_link, html), expected, "{html}");
    }
}
