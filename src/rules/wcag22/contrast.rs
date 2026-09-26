//! WCAG 1.4.3 contrast check. This is a spectrum rule: elements below the configured contrast
//! threshold are reported as warnings rather than errors.
//!
//! Unlike a plain inline-style scan, `color` and the effective `background-color` here come
//! straight from the embedded rendering engine's computed style and cascade resolution, so this
//! applies to every element that renders text, not just ones with an inline `style` attribute.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::{CheckOptions, Finding};

type Rgb = (u8, u8, u8);

const DEFAULT_CANVAS_BACKGROUND: Rgb = (255, 255, 255);
const BLACK: Rgb = (0, 0, 0);
const WHITE: Rgb = (255, 255, 255);

pub fn check_text_contrast(page: &RenderedPage, options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| renders_own_text(*el))
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
    let ratio = color_contrast(foreground, background);
    let threshold = required_threshold(el, options);
    (ratio < threshold).then(|| {
        Finding::warning(
            contrast_rule_id(el),
            format!(
                "<{}> has a text contrast ratio of {ratio:.2}:1, below the required {threshold:.2}:1",
                el.tag()
            ),
        )
        .at(el)
        .help(contrast_help(el, (foreground, background), ratio, options))
    })
}

fn contrast_help(
    el: ElementRef,
    (foreground, background): (Rgb, Rgb),
    ratio: f64,
    options: &CheckOptions,
) -> String {
    let threshold = required_threshold(el, options);
    let fix = match passing_color(foreground, background, threshold) {
        Some((verb, color)) => format!(
            "{verb} the text to {} ({:.2}:1 on {})",
            hex(color),
            color_contrast(color, background),
            hex(background)
        ),
        None => format!(
            "change the background: no text color reaches {threshold:.2}:1 on {}",
            hex(background)
        ),
    };
    let large_threshold = options.large_text_contrast_threshold;
    match !is_large_text(el) && ratio >= large_threshold {
        true => format!(
            "{fix}, or enlarge it to 24px (18.66px bold), which needs only {large_threshold:.2}:1"
        ),
        false => fix,
    }
}

/// The color closest to `foreground`, on the way to black or white (whichever contrasts more
/// with `background`), that reaches `threshold`.
fn passing_color(foreground: Rgb, background: Rgb, threshold: f64) -> Option<(&'static str, Rgb)> {
    let (verb, extreme) =
        match color_contrast(BLACK, background) >= color_contrast(WHITE, background) {
            true => ("darken", BLACK),
            false => ("lighten", WHITE),
        };
    let passes =
        |amount: f64| color_contrast(mix(foreground, extreme, amount), background) >= threshold;
    if !passes(1.0) {
        return None;
    }
    let (mut failing, mut passing) = (0.0, 1.0);
    for _ in 0..32 {
        let middle = (failing + passing) / 2.0;
        match passes(middle) {
            true => passing = middle,
            false => failing = middle,
        }
    }
    Some((verb, mix(foreground, extreme, passing)))
}

fn mix((r1, g1, b1): Rgb, (r2, g2, b2): Rgb, amount: f64) -> Rgb {
    let channel =
        |from: u8, to: u8| (from as f64 + (to as f64 - from as f64) * amount).round() as u8;
    (channel(r1, r2), channel(g1, g2), channel(b1, b2))
}

fn hex((r, g, b): Rgb) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn color_contrast(first: Rgb, second: Rgb) -> f64 {
    contrast_ratio(relative_luminance(first), relative_luminance(second))
}

/// G145 is G18's large-scale text variant, with its lower threshold.
fn contrast_rule_id(el: ElementRef) -> &'static str {
    match is_large_text(el) {
        true => "G145",
        false => "G18",
    }
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

fn relative_luminance((r, g, b): Rgb) -> f64 {
    let channel = |value: u8| -> f64 {
        let normalized = value as f64 / 255.0;
        match normalized <= 0.04045 {
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
    fn help_suggests_the_nearest_passing_color() {
        let p =
            page_from_html(r#"<p style="color: #999999; background-color: #ffffff">Body text</p>"#);
        let findings = check_text_contrast(&p, &CheckOptions::default());
        assert_eq!(
            findings[0].help.as_deref(),
            Some("darken the text to #767676 (4.54:1 on #ffffff)")
        );
    }

    #[test]
    fn help_lightens_text_on_a_dark_background() {
        let p =
            page_from_html(r#"<p style="color: #555555; background-color: #000000">Body text</p>"#);
        let findings = check_text_contrast(&p, &CheckOptions::default());
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with("lighten the text to #"),
            "{:?}",
            findings[0].help
        );
    }

    #[test]
    fn help_offers_large_text_when_that_would_pass() {
        let p =
            page_from_html(r#"<p style="color: #888888; background-color: #ffffff">Body text</p>"#);
        let findings = check_text_contrast(&p, &CheckOptions::default());
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .ends_with("or enlarge it to 24px (18.66px bold), which needs only 3.00:1"),
            "{:?}",
            findings[0].help
        );
    }

    #[test]
    fn help_asks_for_another_background_when_no_text_color_passes() {
        let options = CheckOptions {
            contrast_threshold: 7.0,
            ..CheckOptions::default()
        };
        let p =
            page_from_html(r#"<p style="color: #999999; background-color: #777777">Body text</p>"#);
        let findings = check_text_contrast(&p, &options);
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with("change the background"),
            "{:?}",
            findings[0].help
        );
    }

    #[test]
    fn low_contrast_large_text_is_reported_as_g145() {
        let p = page_from_html(
            r#"<h1 style="color: #aaaaaa; background-color: #ffffff; font-size: 28px">Title</h1>"#,
        );
        let findings = check_text_contrast(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "G145");
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
