//! WCAG 3.1.1 check for the page's declared language.

use crate::page::RenderedPage;

use super::{CheckOptions, Finding};

/// H57: the `html` element must declare a non-empty `lang` attribute.
pub fn check_missing_lang(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    match has_lang(page) {
        true => Vec::new(),
        false => vec![Finding::error(
            "H57",
            "<html> element has no non-empty lang attribute".to_string(),
        )],
    }
}

fn has_lang(page: &RenderedPage) -> bool {
    page.by_tag("html")
        .next()
        .and_then(|html| html.attr("lang"))
        .is_some_and(|lang| !lang.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn missing_lang_is_flagged() {
        let p = page_from_html("<html><head></head><body></body></html>");
        let findings = check_missing_lang(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H57");
    }

    #[test]
    fn empty_lang_is_flagged() {
        let p = page_from_html(r#"<html lang=""><body></body></html>"#);
        assert_eq!(check_missing_lang(&p, &CheckOptions::default()).len(), 1);
    }

    #[test]
    fn present_lang_is_not_flagged() {
        let p = page_from_html(r#"<html lang="en"><body></body></html>"#);
        assert!(check_missing_lang(&p, &CheckOptions::default()).is_empty());
    }
}
