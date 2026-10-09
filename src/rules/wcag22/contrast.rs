//! WCAG 1.4.3 contrast check. This is a spectrum rule: elements below the configured contrast
//! threshold are reported as warnings rather than errors.
//!
//! Unlike a plain inline-style scan, `color` and the effective `background-color` here come
//! straight from the embedded rendering engine's computed style and cascade resolution, so this
//! applies to every element that renders text, not just ones with an inline `style` attribute.

use crate::page::{ElementRef, RenderedPage, Rgb};

use crate::rules::color::{
    DEFAULT_CANVAS_BACKGROUND, color_contrast, fix_suggestion, is_large_text, required_threshold,
};
use crate::rules::{CheckOptions, Finding};

pub fn check_text_contrast(page: &RenderedPage, options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| el.has_own_text())
        .filter_map(|el| contrast_finding(el, options))
        .collect()
}

fn contrast_finding(el: ElementRef, options: &CheckOptions) -> Option<Finding> {
    let foreground = el.color();
    let background = el.background_color().unwrap_or(DEFAULT_CANVAS_BACKGROUND);
    let ratio = color_contrast(foreground, background);
    let threshold = required_threshold(el, options);
    (ratio < threshold).then(|| {
        Finding::warning_at(
            el,
            contrast_rule_id(el),
            format!(
                "<{}> has a text contrast ratio of {ratio:.2}:1, below the required {threshold:.2}:1",
                el.tag()
            ),
        )
        .help(contrast_help(el, (foreground, background), ratio, options))
    })
}

fn contrast_help(
    el: ElementRef,
    (foreground, background): (Rgb, Rgb),
    ratio: f64,
    options: &CheckOptions,
) -> String {
    let fix = fix_suggestion(foreground, background, required_threshold(el, options));
    let large_threshold = options.large_text_contrast_threshold;
    match !is_large_text(el) && ratio >= large_threshold {
        true => format!(
            "{fix}, or enlarge it to 24px (18.66px bold), which needs only {large_threshold:.2}:1"
        ),
        false => fix,
    }
}

/// G145 is G18's large-scale text variant, with its lower threshold.
fn contrast_rule_id(el: ElementRef) -> &'static str {
    match is_large_text(el) {
        true => "G145",
        false => "G18",
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;
    use crate::rules::testutil::{assert_one, run_with_defaults};

    #[test]
    fn low_contrast_text_is_flagged_as_warning() {
        let html = r#"<p style="color: #999999; background-color: #ffffff">Body text</p>"#;
        let finding = assert_one(run_with_defaults(check_text_contrast, html), "G18");
        assert_eq!(finding.severity, crate::rules::Severity::Warning);
    }

    #[test]
    fn help_suggests_the_nearest_passing_color() {
        let html = r#"<p style="color: #999999; background-color: #ffffff">Body text</p>"#;
        let findings = run_with_defaults(check_text_contrast, html);
        assert_eq!(
            findings[0].help.as_deref(),
            Some("darken the text to #767676 (4.54:1 on #ffffff)")
        );
    }

    #[test]
    fn help_lightens_text_on_a_dark_background() {
        let html = r#"<p style="color: #555555; background-color: #000000">Body text</p>"#;
        let findings = run_with_defaults(check_text_contrast, html);
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
        let html = r#"<p style="color: #888888; background-color: #ffffff">Body text</p>"#;
        let findings = run_with_defaults(check_text_contrast, html);
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

    #[rstest]
    #[case(
        r#"<h1 style="color: #aaaaaa; background-color: #ffffff; font-size: 28px">Title</h1>"#,
        &["G145"]
    )]
    #[case(r#"<p style="color: #000000; background-color: #ffffff">Body text</p>"#, &[])]
    // Large bold text uses the lower threshold.
    #[case(
        r#"<h1 style="color: #949494; background-color: #ffffff; font-size: 28px; font-weight: bold">Title</h1>"#,
        &[]
    )]
    // Only the span with direct text is flagged, not the div with no text children.
    #[case(
        r#"<div style="color: #999999; background-color: #ffffff"><span style="color: #999999; background-color: #ffffff">nested</span></div>"#,
        &["G18"]
    )]
    // A missing background defaults to the white canvas: light gray still fails.
    #[case(r#"<p style="color: #eeeeee">Body text</p>"#, &["G18"])]
    fn text_contrast(#[case] html: &str, #[case] expected: &[&str]) {
        let findings = run_with_defaults(check_text_contrast, html);
        let rule_ids: Vec<_> = findings.iter().map(|finding| finding.rule_id).collect();
        assert_eq!(rule_ids, expected, "{html}");
    }
}
