//! WCAG 2.4.1 check for a "skip to main content" link.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

/// G1: a page should offer a link (usually the first focusable element) that jumps past repeated
/// navigation straight to the main content, and its target must actually exist.
pub fn check_missing_skip_link(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
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
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn dangling_skip_link_help_names_the_missing_id() {
        let p = page_from_html(r##"<a href="#content">Skip to content</a><main></main>"##);
        let findings = check_missing_skip_link(&p, &CheckOptions::default());
        assert_eq!(
            findings[0].help.as_deref(),
            Some(r#"add id="content" to the main content, e.g. <main id="content">"#)
        );
    }

    #[test]
    fn missing_skip_link_is_flagged() {
        let p = page_from_html(r#"<body><nav></nav><main id="main"></main></body>"#);
        let findings = check_missing_skip_link(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "G1");
    }

    #[test]
    fn valid_skip_link_is_not_flagged() {
        let p = page_from_html(
            r##"<body><a href="#main">Skip to main content</a><main id="main"></main></body>"##,
        );
        assert!(check_missing_skip_link(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn dangling_skip_link_target_is_flagged() {
        let p = page_from_html(r##"<body><a href="#main">Skip to main content</a></body>"##);
        let findings = check_missing_skip_link(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "G1");
    }
}
