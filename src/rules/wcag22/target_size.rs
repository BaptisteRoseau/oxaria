//! WCAG 2.5.8 (Target Size Minimum, new in 2.2) check. This is a spectrum rule: a measured
//! target below the configured threshold is reported as a warning.
//!
//! Uses the embedded rendering engine's real layout box for each interactive element, so this
//! applies regardless of how the element's size was actually set (inline style, CSS class,
//! padding around intrinsic content, ...), not just elements with an inline `style` attribute.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::{CheckOptions, Finding};

fn is_interactive(el: ElementRef) -> bool {
    (el.tag() == "a" && el.has_attr("href"))
        || el.tag() == "button"
        || el.attr("role") == Some("button")
        || (el.tag() == "input"
            && matches!(
                el.attr("type"),
                Some("button" | "submit" | "checkbox" | "radio" | "image")
            ))
}

/// Content litehtml can't size: it never loads images, ignores `<svg>`'s
/// `width`/`height`, and has no intrinsic size for form controls.
const UNSIZED_CONTENT_TAGS: &[&str] = &["img", "svg", "picture", "video", "canvas"];

/// The smallest dimension that was actually measured, if any.
fn measured_size(element: ElementRef) -> Option<f32> {
    let bbox = element.bounding_box();
    match bbox.width == 0.0 && bbox.height == 0.0 {
        // Not laid out (e.g. `display: none`), or -- in hand-built test fixtures -- no size was
        // ever declared. Either way there is nothing to measure.
        true => None,
        // A 0 on one axis means "not measured", not 0px.
        false if has_unsized_content(element) => [bbox.width, bbox.height]
            .into_iter()
            .filter(|dimension| *dimension > 0.0)
            .reduce(f32::min),
        false => Some(bbox.smallest_dimension()),
    }
}

fn has_unsized_content(element: ElementRef) -> bool {
    element.tag() == "input"
        || (element.text().is_empty()
            && element
                .descendants()
                .any(|child| UNSIZED_CONTENT_TAGS.contains(&child.tag())))
}

pub fn check_target_size(page: &RenderedPage, options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_interactive(*el))
        .filter_map(|element| target_size_finding(element, options))
        .collect()
}

fn target_size_finding(element: ElementRef, options: &CheckOptions) -> Option<Finding> {
    let smallest_dimension = measured_size(element)? as f64;
    (smallest_dimension < options.target_size_threshold).then(|| {
        Finding::warning(
            "TGT001",
            format!(
                "<{}> target size is {smallest_dimension:.0}px, below the required {:.0}px",
                element.tag(),
                options.target_size_threshold
            ),
        )
        .at(element)
        .help(target_size_help(
            element,
            smallest_dimension,
            options.target_size_threshold,
        ))
    })
}

fn target_size_help(element: ElementRef, size: f64, threshold: f64) -> String {
    let padding = ((threshold - size) / 2.0).ceil();
    let fix = format!(
        "make it at least {threshold:.0}x{threshold:.0}px, e.g. min-width: {threshold:.0}px; \
         min-height: {threshold:.0}px, or padding: {padding:.0}px"
    );
    match element.tag() {
        "a" => format!("{fix} (a link inside a sentence is exempt)"),
        _ => fix,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;
    use crate::rules::testutil::{assert_one, run_with_defaults};

    #[test]
    fn help_gives_the_missing_padding() {
        let html = r#"<button style="width: 19px; height: 19px">X</button>"#;
        let findings = run_with_defaults(check_target_size, html);
        assert_eq!(
            findings[0].help.as_deref(),
            Some(
                "make it at least 24x24px, e.g. min-width: 24px; min-height: 24px, or padding: 3px"
            )
        );
    }

    #[test]
    fn link_help_mentions_the_inline_exception() {
        let html = r#"<a href="/" style="width: 16px; height: 16px">X</a>"#;
        let findings = run_with_defaults(check_target_size, html);
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .ends_with("(a link inside a sentence is exempt)")
        );
    }

    #[test]
    fn small_target_is_flagged_as_warning() {
        let html = r#"<button style="width: 16px; height: 16px">X</button>"#;
        let finding = assert_one(run_with_defaults(check_target_size, html), "TGT001");
        assert_eq!(finding.severity, crate::rules::Severity::Warning);
    }

    #[test]
    fn custom_threshold_is_respected() {
        let opts = CheckOptions {
            target_size_threshold: 44.0,
            ..CheckOptions::default()
        };
        let p = page_from_html(r#"<button style="width: 32px; height: 32px">X</button>"#);
        assert_eq!(check_target_size(&p, &opts).len(), 1);
    }

    #[rstest]
    #[case(r#"<button style="width: 24px; height: 24px">X</button>"#, 0)]
    // An element without a size is skipped.
    #[case("<button>X</button>", 0)]
    // What litehtml produces for `.lsb { height: 30px }` on google.com's submit buttons: measured
    // by that height.
    #[case(
        r#"<input type="submit" value="Search" style="width: 0px; height: 30px">"#,
        0
    )]
    // gitlab.com's customer logo links: CSS width 164px, height 0 because the `<img>` inside is
    // never loaded.
    #[case(
        r#"<a href="/customers/weave/" aria-label="weave logo" style="width: 164px; height: 0px"><img src="logo.png"></a>"#,
        0
    )]
    // A text link containing an icon is measured normally.
    #[case(
        r#"<a href="/x" style="width: 0px; height: 30px">Docs <svg></svg></a>"#,
        1
    )]
    // Only form controls lack an intrinsic size; a laid-out 0px-wide link is a real finding.
    #[case(r#"<a href="/x" style="width: 0px; height: 30px">X</a>"#, 1)]
    // An anchor without href is not checked.
    #[case(r#"<a name="top" style="width: 4px; height: 4px"></a>"#, 0)]
    #[case(r#"<div style="width: 4px; height: 4px"></div>"#, 0)]
    fn target_size(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(
            run_with_defaults(check_target_size, html).len(),
            expected,
            "{html}"
        );
    }

    #[rstest]
    #[case(r#"<input type="submit" value="Search" style="width: 0px; height: 16px">"#)]
    // docker.com's search button: an `<svg>` icon, 16px high, 0 wide.
    #[case(
        r#"<button aria-label="Search" style="width: 0px; height: 16px"><svg viewBox="0 0 24 24"></svg></button>"#
    )]
    fn target_with_only_a_small_css_height_is_measured_by_that_height(#[case] html: &str) {
        let findings = run_with_defaults(check_target_size, html);
        assert_eq!(findings.len(), 1, "{html}");
        assert!(
            findings[0].message.contains("16px"),
            "{}",
            findings[0].message
        );
    }
}
