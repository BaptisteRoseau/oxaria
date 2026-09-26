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
        .map(|lang| !lang.trim().is_empty())
        .unwrap_or(false)
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
    fn missing_lang_is_flagged() {
        let p = page_from_html("<html><head></head><body></body></html>");
        let findings = check_missing_lang(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H57");
    }

    #[test]
    fn empty_lang_is_flagged() {
        let p = page_from_html(r#"<html lang=""><body></body></html>"#);
        assert_eq!(check_missing_lang(&p, &options()).len(), 1);
    }

    #[test]
    fn present_lang_is_not_flagged() {
        let p = page_from_html(r#"<html lang="en"><body></body></html>"#);
        assert!(check_missing_lang(&p, &options()).is_empty());
    }
}
