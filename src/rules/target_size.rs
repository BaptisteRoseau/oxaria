//! WCAG 2.5.8 (Target Size Minimum, new in 2.2) check. This is a spectrum rule: a measured
//! target below the configured threshold is reported as a warning.
//!
//! Uses the embedded rendering engine's real layout box for each interactive element, so this
//! applies regardless of how the element's size was actually set (inline style, CSS class,
//! padding around intrinsic content, ...), not just elements with an inline `style` attribute.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

fn is_interactive(el: ElementRef) -> bool {
    matches!(el.tag(), "a" | "button")
        || el.attr("role") == Some("button")
        || (el.tag() == "input"
            && matches!(
                el.attr("type"),
                Some("button") | Some("submit") | Some("checkbox") | Some("radio") | Some("image")
            ))
}

pub fn check_target_size(page: &RenderedPage, options: &CheckOptions) -> Vec<Finding> {
    page.select(is_interactive)
        .into_iter()
        .filter_map(|element| target_size_finding(element, options))
        .collect()
}

fn target_size_finding(element: ElementRef, options: &CheckOptions) -> Option<Finding> {
    let bbox = element.bounding_box();
    if bbox.width == 0.0 && bbox.height == 0.0 {
        // Not laid out (e.g. `display: none`), or -- in hand-built test fixtures -- no size was
        // ever declared. Either way there is nothing to measure.
        return None;
    }
    let smallest_dimension = bbox.smallest_dimension() as f64;
    (smallest_dimension < options.target_size_threshold).then(|| {
        Finding::warning(
            "TGT001",
            format!(
                "<{}> target size is {smallest_dimension:.0}px, below the required {:.0}px",
                element.tag(),
                options.target_size_threshold
            ),
        )
    })
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
    fn small_target_is_flagged_as_warning() {
        let p = page_from_html(r#"<button style="width: 16px; height: 16px">X</button>"#);
        let findings = check_target_size(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "TGT001");
        assert_eq!(findings[0].severity, crate::rules::Severity::Warning);
    }

    #[test]
    fn large_target_is_not_flagged() {
        let p = page_from_html(r#"<button style="width: 24px; height: 24px">X</button>"#);
        assert!(check_target_size(&p, &options()).is_empty());
    }

    #[test]
    fn element_without_size_is_skipped() {
        let p = page_from_html("<button>X</button>");
        assert!(check_target_size(&p, &options()).is_empty());
    }

    #[test]
    fn custom_threshold_is_respected() {
        let mut opts = options();
        opts.target_size_threshold = 44.0;
        let p = page_from_html(r#"<button style="width: 32px; height: 32px">X</button>"#);
        assert_eq!(check_target_size(&p, &opts).len(), 1);
    }

    #[test]
    fn non_interactive_element_is_not_checked() {
        let p = page_from_html(r#"<div style="width: 4px; height: 4px"></div>"#);
        assert!(check_target_size(&p, &options()).is_empty());
    }
}
