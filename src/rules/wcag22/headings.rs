//! WCAG 1.3.1 / 2.4.6 checks for heading structure and semantics.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

/// H42/G141: the page should have a top-level heading to anchor its outline.
pub fn check_missing_h1(page: &RenderedPage) -> Vec<Finding> {
    match page.by_tag("h1").next().is_none() {
        true => vec![Finding::error("H42", "page has no <h1> element").help(
            "mark the page's main title up as an <h1>, e.g. <h1>Order summary</h1>, \
                 instead of styling a <div> or <p> to look like one",
        )],
        false => Vec::new(),
    }
}

/// G141: heading levels should nest sequentially; jumping forward more than one level (e.g. h2
/// straight to h4) breaks the document outline for assistive technology.
pub fn check_skipped_heading_level(page: &RenderedPage) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut previous_level = 0u8;
    for (heading, level) in headings(page) {
        if previous_level > 0 && level > previous_level + 1 {
            findings.push(
                Finding::error_at(
                    heading,
                    "G141",
                    skipped_level_message(previous_level, level),
                )
                .help(skipped_level_help(previous_level)),
            );
        }
        previous_level = level;
    }
    findings
}

/// F92: `role="presentation"` (or `none`) strips a heading's semantics, dropping it from the
/// outline that assistive technology users navigate by.
pub fn check_presentational_heading(page: &RenderedPage) -> Vec<Finding> {
    headings(page)
        .into_iter()
        .filter(|(heading, _)| matches!(heading.attr("role"), Some("presentation" | "none")))
        .map(|(heading, level)| {
            Finding::error_at(
                heading,
                "F92",
                format!(
                    "<h{level}> has role=\"{}\", hiding that it is a heading",
                    heading.attr("role").unwrap_or_default()
                ),
            )
            .help(format!(
                "remove the role; if the text is not a heading, use a <p> or <div> instead of <h{level}>"
            ))
        })
        .collect()
}

fn headings(page: &RenderedPage) -> Vec<(ElementRef<'_>, u8)> {
    page.all()
        .filter_map(|el| heading_level(el.tag()).map(|level| (el, level)))
        .collect()
}

fn heading_level(tag_name: &str) -> Option<u8> {
    tag_name.strip_prefix('h')?.parse().ok()
}

fn skipped_level_help(previous_level: u8) -> String {
    let expected = previous_level + 1;
    format!("make this an <h{expected}>; to only make it smaller, change its font-size in CSS")
}

fn skipped_level_message(previous_level: u8, level: u8) -> String {
    format!("heading level jumps from h{previous_level} to h{level}, skipping a level")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::{help, rule_ids};

    #[test]
    fn skipped_level_help_names_the_expected_level() {
        assert!(
            help(check_skipped_heading_level, "<h1>A</h1><h3>B</h3>")
                .starts_with("make this an <h2>")
        );
    }

    #[rstest]
    #[case("<h2>Section</h2>", &["H42"])]
    #[case("<h1>Title</h1>", &[])]
    fn missing_h1(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_missing_h1, html), expected, "{html}");
    }

    #[rstest]
    #[case("<h1>T</h1><h4>Specifications</h4>", &["G141"])]
    #[case("<h1>T</h1><h2>A</h2><h2>B</h2><h3>C</h3>", &[])]
    // Going back up a level is not a skip.
    #[case("<h1>T</h1><h2>A</h2><h3>B</h3><h2>C</h2>", &[])]
    fn skipped_heading_level(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(
            rule_ids(check_skipped_heading_level, html),
            expected,
            "{html}"
        );
    }

    #[rstest]
    #[case(
        r#"<h1>T</h1><h2 role="presentation">Reviews</h2><h3 role="none">A</h3>"#,
        &["F92", "F92"]
    )]
    #[case(r#"<h2 role="heading">Reviews</h2><h3>Specs</h3>"#, &[])]
    fn presentational_heading(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(
            rule_ids(check_presentational_heading, html),
            expected,
            "{html}"
        );
    }
}
