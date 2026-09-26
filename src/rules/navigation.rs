//! WCAG 2.4.1 check for a "skip to main content" link.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

/// G1: a page should offer a link (usually the first focusable element) that jumps past repeated
/// navigation straight to the main content, and its target must actually exist.
pub fn check_missing_skip_link(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    match find_skip_link(page) {
        None => vec![Finding::error(
            "G1",
            "no skip link found (an <a href=\"#...\"> whose text mentions \"skip\")".to_string(),
        )],
        Some(link) if !target_exists(page, link) => {
            vec![Finding::error("G1", dangling_skip_link_message(link)).at(link)]
        }
        Some(_) => Vec::new(),
    }
}

fn find_skip_link(page: &RenderedPage) -> Option<ElementRef<'_>> {
    page.select(is_fragment_link)
        .into_iter()
        .find(|link| link.text().to_lowercase().contains("skip"))
}

fn is_fragment_link(el: ElementRef) -> bool {
    el.tag() == "a" && el.attr("href").is_some_and(|href| href.starts_with('#'))
}

fn target_exists(page: &RenderedPage, link: ElementRef) -> bool {
    match fragment_id(link) {
        Some(id) => page.element_by_id(id).is_some(),
        None => false,
    }
}

fn fragment_id(link: ElementRef<'_>) -> Option<&str> {
    link.attr("href")?.strip_prefix('#')
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
