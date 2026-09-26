//! WCAG 1.3.1 / 2.4.6 checks for heading structure and semantics.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::{CheckOptions, Finding};

/// H42/G141: the page should have a top-level heading to anchor its outline.
pub fn check_missing_h1(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    match page.by_tag("h1").next().is_none() {
        true => vec![
            Finding::error("H42", "page has no <h1> element".to_string()).help(
                "mark the page's main title up as an <h1>, e.g. <h1>Order summary</h1>, \
                 instead of styling a <div> or <p> to look like one",
            ),
        ],
        false => Vec::new(),
    }
}

/// G141: heading levels should nest sequentially; jumping forward more than one level (e.g. h2
/// straight to h4) breaks the document outline for assistive technology.
pub fn check_skipped_heading_level(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut previous_level = 0u8;
    for (heading, level) in headings(page) {
        if previous_level > 0 && level > previous_level + 1 {
            findings.push(
                Finding::error("G141", skipped_level_message(previous_level, level))
                    .at(heading)
                    .help(skipped_level_help(previous_level)),
            );
        }
        previous_level = level;
    }
    findings
}

/// F92: `role="presentation"` (or `none`) strips a heading's semantics, dropping it from the
/// outline that assistive technology users navigate by.
pub fn check_presentational_heading(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    headings(page)
        .into_iter()
        .filter(|(heading, _)| matches!(heading.attr("role"), Some("presentation" | "none")))
        .map(|(heading, level)| {
            Finding::error(
                "F92",
                format!(
                    "<h{level}> has role=\"{}\", hiding that it is a heading",
                    heading.attr("role").unwrap_or_default()
                ),
            )
            .at(heading)
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
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn skipped_level_help_names_the_expected_level() {
        let p = page_from_html("<h1>A</h1><h3>B</h3>");
        let findings = check_skipped_heading_level(&p, &CheckOptions::default());
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with("make this an <h2>")
        );
    }

    #[test]
    fn missing_h1_is_flagged() {
        let p = page_from_html("<h2>Section</h2>");
        let findings = check_missing_h1(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H42");
    }

    #[test]
    fn present_h1_is_not_flagged() {
        let p = page_from_html("<h1>Title</h1>");
        assert!(check_missing_h1(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn sequential_headings_are_not_flagged() {
        let p = page_from_html("<h1>T</h1><h2>A</h2><h2>B</h2><h3>C</h3>");
        assert!(check_skipped_heading_level(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn skipped_heading_level_is_flagged() {
        let p = page_from_html("<h1>T</h1><h4>Specifications</h4>");
        let findings = check_skipped_heading_level(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "G141");
    }

    #[test]
    fn heading_going_backwards_is_not_flagged() {
        let p = page_from_html("<h1>T</h1><h2>A</h2><h3>B</h3><h2>C</h2>");
        assert!(check_skipped_heading_level(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn presentational_heading_is_flagged() {
        let p = page_from_html(
            r#"<h1>T</h1><h2 role="presentation">Reviews</h2><h3 role="none">A</h3>"#,
        );
        let findings = check_presentational_heading(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].rule_id, "F92");
    }

    #[test]
    fn heading_with_other_role_is_not_flagged() {
        let p = page_from_html(r#"<h2 role="heading">Reviews</h2><h3>Specs</h3>"#);
        assert!(check_presentational_heading(&p, &CheckOptions::default()).is_empty());
    }
}
