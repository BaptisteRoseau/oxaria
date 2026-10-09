//! WCAG 3.1.1 check for the page's declared language.

use crate::page::RenderedPage;

use crate::rules::Finding;

/// H57: the `html` element must declare a non-empty `lang` attribute.
pub fn check_missing_lang(page: &RenderedPage) -> Vec<Finding> {
    match has_lang(page) {
        true => Vec::new(),
        false => vec![
            Finding::error(
                "H57",
                "<html> element has no non-empty lang attribute".to_string(),
            )
            .help("declare the page's main language, e.g. <html lang=\"en\">"),
        ],
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
    use crate::rules::testutil::{assert_fires, count, run};

    #[test]
    fn missing_lang_is_flagged() {
        let html = "<html><head></head><body></body></html>";
        assert_fires(check_missing_lang, html, "H57");
    }

    #[test]
    fn empty_lang_is_flagged() {
        let html = r#"<html lang=""><body></body></html>"#;
        assert_eq!(count(check_missing_lang, html), 1);
    }

    #[test]
    fn present_lang_is_not_flagged() {
        let html = r#"<html lang="en"><body></body></html>"#;
        assert!(run(check_missing_lang, html).is_empty());
    }
}
