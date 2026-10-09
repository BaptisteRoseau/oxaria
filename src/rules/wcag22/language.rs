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
        .and_then(|html| html.non_empty_attr("lang"))
        .is_some()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::rule_ids;

    #[rstest]
    #[case("<html><head></head><body></body></html>", &["H57"])]
    #[case(r#"<html lang=""><body></body></html>"#, &["H57"])]
    #[case(r#"<html lang="en"><body></body></html>"#, &[])]
    fn missing_lang(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_missing_lang, html), expected, "{html}");
    }
}
