//! WCAG 1.4.3 contrast check. This is a spectrum rule: elements below the configured contrast
//! threshold are reported as warnings rather than errors.
//!
//! Unlike a plain inline-style scan, `color` and the effective `background-color` here come
//! straight from the embedded rendering engine's computed style and cascade resolution, so this
//! applies to every element that renders text, not just ones with an inline `style` attribute.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

const DEFAULT_CANVAS_BACKGROUND: (u8, u8, u8) = (255, 255, 255);

pub fn check_text_contrast(page: &RenderedPage, options: &CheckOptions) -> Vec<Finding> {
    page.select(renders_own_text)
        .into_iter()
        .filter_map(|el| contrast_finding(el, options))
        .collect()
}

fn renders_own_text(el: ElementRef) -> bool {
    el.children()
        .any(|child| child.node().is_text() && !child.node().own_text.trim().is_empty())
}

fn contrast_finding(el: ElementRef, options: &CheckOptions) -> Option<Finding> {
    let foreground = el.color();
    let background = el.background_color().unwrap_or(DEFAULT_CANVAS_BACKGROUND);
    let ratio = contrast_ratio(
        relative_luminance(foreground),
        relative_luminance(background),
    );
    let threshold = required_threshold(el, options);
    (ratio < threshold).then(|| {
        Finding::warning(
            "G18",
            format!(
                "<{}> has a text contrast ratio of {ratio:.2}:1, below the required {threshold:.2}:1",
                el.tag()
            ),
        )
        .at(el)
    })
}

fn required_threshold(el: ElementRef, options: &CheckOptions) -> f64 {
    match is_large_text(el) {
        true => options.large_text_contrast_threshold,
        false => options.contrast_threshold,
    }
}

/// WCAG defines "large scale" text as at least 18pt (24px), or at least 14pt (18.66px) bold.
fn is_large_text(el: ElementRef) -> bool {
    let size = el.font_size_px();
    size >= 24.0 || (el.font_weight() >= 700 && size >= 18.66)
}

fn relative_luminance((r, g, b): (u8, u8, u8)) -> f64 {
    let channel = |value: u8| -> f64 {
        let normalized = value as f64 / 255.0;
        match normalized <= 0.03928 {
            true => normalized / 12.92,
            false => ((normalized + 0.055) / 1.055).powf(2.4),
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let (lighter, darker) = match l1 > l2 {
        true => (l1, l2),
        false => (l2, l1),
    };
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn relative_luminance_black_is_zero() {
        assert!((relative_luminance((0, 0, 0)) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn relative_luminance_white_is_one() {
        assert!((relative_luminance((255, 255, 255)) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn contrast_ratio_black_on_white_is_twenty_one() {
        let ratio = contrast_ratio(
            relative_luminance((0, 0, 0)),
            relative_luminance((255, 255, 255)),
        );
        assert!((ratio - 21.0).abs() < 0.01);
    }

    #[test]
    fn low_contrast_text_is_flagged_as_warning() {
        let p =
            page_from_html(r#"<p style="color: #999999; background-color: #ffffff">Body text</p>"#);
        let findings = check_text_contrast(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "G18");
        assert_eq!(findings[0].severity, crate::rules::Severity::Warning);
    }

    #[test]
    fn high_contrast_text_is_not_flagged() {
        let p =
            page_from_html(r#"<p style="color: #000000; background-color: #ffffff">Body text</p>"#);
        assert!(check_text_contrast(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn large_bold_text_uses_lower_threshold() {
        let p = page_from_html(
            r#"<h1 style="color: #949494; background-color: #ffffff; font-size: 28px; font-weight: bold">Title</h1>"#,
        );
        assert!(check_text_contrast(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn element_with_no_text_children_is_skipped() {
        let p = page_from_html(
            r#"<div style="color: #999999; background-color: #ffffff"><span style="color: #999999; background-color: #ffffff">nested</span></div>"#,
        );
        let findings = check_text_contrast(&p, &CheckOptions::default());
        assert_eq!(
            findings.len(),
            1,
            "only the span with direct text should be flagged"
        );
    }

    #[test]
    fn missing_background_defaults_to_white_canvas() {
        let p = page_from_html(r#"<p style="color: #eeeeee">Body text</p>"#);
        let findings = check_text_contrast(&p, &CheckOptions::default());
        assert_eq!(
            findings.len(),
            1,
            "light gray on default white should still fail"
        );
    }
}
