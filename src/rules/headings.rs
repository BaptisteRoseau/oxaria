//! WCAG 1.3.1 / 2.4.6 checks for heading structure.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

/// H42/G141: the page should have exactly one top-level heading to anchor its outline.
pub fn check_missing_h1(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    match page.by_tag("h1").next().is_none() {
        true => vec![Finding::error(
            "H42",
            "page has no <h1> element".to_string(),
        )],
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
                Finding::error("G141", skipped_level_message(previous_level, level)).at(heading),
            );
        }
        previous_level = level;
    }
    findings
}

fn headings(page: &RenderedPage) -> Vec<(ElementRef<'_>, u8)> {
    page.all()
        .filter_map(|el| heading_level(el.tag()).map(|level| (el, level)))
        .collect()
}

fn heading_level(tag_name: &str) -> Option<u8> {
    tag_name.strip_prefix('h')?.parse().ok()
}

fn skipped_level_message(previous_level: u8, level: u8) -> String {
    format!("heading level jumps from h{previous_level} to h{level}, skipping a level")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

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
}
