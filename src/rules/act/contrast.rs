//! ACT text contrast rule. A spectrum rule, like G18: findings are warnings.
//! The contrast computation is WCAG's (`wcag22::contrast`); what ACT adds is
//! its applicability, which leaves out text of disabled widgets and their
//! labels.

use std::collections::HashSet;

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{effective_role, is_aria_true, is_disabled, is_not_rendered};
use crate::rules::wcag22::{
    DEFAULT_CANVAS_BACKGROUND, Rgb, color_contrast, hex, is_large_text, passing_color,
    renders_own_text,
};
use crate::rules::{CheckOptions, Finding};

/// afw4f7: text needs a contrast ratio of at least 4.5:1, or 3:1 when large
/// (the configured Level AA thresholds).
pub fn check_contrast_minimum(page: &RenderedPage, options: &CheckOptions) -> Vec<Finding> {
    let disabled_label_ids = disabled_widget_label_ids(page);
    page.all()
        .filter(|el| renders_own_text(*el) && !is_not_rendered(*el))
        .filter(|el| !is_in_disabled_widget(*el) && !is_in_disabled_label(*el, &disabled_label_ids))
        .filter_map(|el| contrast_finding(el, options))
        .collect()
}

fn contrast_finding(el: ElementRef, options: &CheckOptions) -> Option<Finding> {
    let foreground = el.color();
    let background = el.background_color().unwrap_or(DEFAULT_CANVAS_BACKGROUND);
    let ratio = color_contrast(foreground, background);
    let threshold = match is_large_text(el) {
        true => options.large_text_contrast_threshold,
        false => options.contrast_threshold,
    };
    (ratio < threshold).then(|| {
        Finding::warning(
            "afw4f7",
            format!(
                "<{}> text {} on {} has a contrast ratio of {ratio:.2}:1, below {threshold:.2}:1",
                el.tag(),
                hex(foreground),
                hex(background)
            ),
        )
        .at(el)
        .help(contrast_help(foreground, background, threshold))
    })
}

fn contrast_help(foreground: Rgb, background: Rgb, threshold: f64) -> String {
    match passing_color(foreground, background, threshold) {
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
    }
}

/// Inside a disabled `group` or widget: native `disabled`, or
/// `aria-disabled="true"` on such a role.
fn is_in_disabled_widget(el: ElementRef) -> bool {
    std::iter::once(el).chain(el.ancestors()).any(|el| {
        is_disabled(el)
            || (is_aria_true(el, "aria-disabled")
                && effective_role(el).is_some_and(|role| role.is_a("group") || role.is_a("widget")))
    })
}

/// Inside the `label` of a disabled control, or an element naming a
/// disabled widget through `aria-labelledby`.
fn is_in_disabled_label(el: ElementRef, disabled_label_ids: &HashSet<&str>) -> bool {
    std::iter::once(el).chain(el.ancestors()).any(|el| {
        (el.tag() == "label" && labels_disabled_control(el))
            || el
                .attr("id")
                .is_some_and(|id| disabled_label_ids.contains(id))
    })
}

fn labels_disabled_control(label: ElementRef) -> bool {
    let wrapped = label.descendants().any(is_disabled_control);
    let targeted = label
        .attr("for")
        .and_then(|id| label.page().element_by_id(id))
        .is_some_and(is_disabled_control);
    wrapped || targeted
}

fn is_disabled_control(el: ElementRef) -> bool {
    matches!(el.tag(), "button" | "input" | "select" | "textarea") && is_in_disabled_widget(el)
}

/// The ids that disabled widgets point `aria-labelledby` at.
fn disabled_widget_label_ids(page: &RenderedPage) -> HashSet<&str> {
    page.all()
        .filter_map(|widget| Some((widget, widget.attr("aria-labelledby")?)))
        .filter(|(widget, _)| is_in_disabled_widget(*widget))
        .flat_map(|(_, ids)| ids.split_ascii_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;
    use crate::rules::{RuleCheck, Severity};

    fn findings(check: RuleCheck, html: &str) -> Vec<Finding> {
        check(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(
        r#"<p style="color: #AAAAAA; background-color: white">Some text in English</p>"#,
        1
    )]
    #[case(
        r#"<button style="color: #777777; background-color: #EEEEEE">My button!</button>"#,
        1
    )]
    #[case(
        r#"<div role="button" style="color: #777777; background-color: #EEEEEE">My button!</div>"#,
        1
    )]
    #[case(
        r#"<p style="color: #333333; background-color: #FFFFFF">Some text</p>"#,
        0
    )]
    #[case(
        r#"<p style="color: #000000; font-size: 24px; background-color: #666666">Large text</p>"#,
        0
    )]
    #[case(r#"<p style="color: #000000; font-size: 19px; font-weight: 700; background-color: #666666">Bold</p>"#, 0)]
    #[case("<p>Default colors</p>", 0)]
    #[case(r#"<p style="color: #AAAAAA" hidden>Invisible</p>"#, 0)]
    #[case(
        r#"<label style="color: #888888">My name <input type="text" disabled></label>"#,
        0
    )]
    #[case(r#"<label id="n" style="color: #888888">Pet</label><div role="textbox" aria-labelledby="n" aria-disabled="true"></div>"#, 0)]
    #[case(
        r#"<fieldset disabled><label style="color: #888888">My name <input></label></fieldset>"#,
        0
    )]
    #[case(r#"<div role="group" aria-disabled="true"><label style="color: #888888">Name <input></label></div>"#, 0)]
    #[case(
        r#"<button style="color: #777777; background-color: #EEEEEE" disabled>My button!</button>"#,
        0
    )]
    #[case(r#"<div role="button" style="color: #777777; background-color: #EEEEEE" aria-disabled="true">Mine</div>"#, 0)]
    fn contrast_minimum_afw4f7(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(
            findings(check_contrast_minimum, html).len(),
            expected,
            "{html}"
        );
    }

    #[test]
    fn findings_are_warnings_quoting_the_colors() {
        let found = findings(
            check_contrast_minimum,
            r#"<p style="color: #999999; background-color: #ffffff">Body text</p>"#,
        );
        assert_eq!(found[0].severity, Severity::Warning);
        assert_eq!(
            found[0].message,
            "<p> text #999999 on #ffffff has a contrast ratio of 2.85:1, below 4.50:1"
        );
        assert_eq!(
            found[0].help.as_deref(),
            Some("darken the text to #767676 (4.54:1 on #ffffff)")
        );
    }

    #[test]
    fn minimum_threshold_follows_the_options() {
        let options = CheckOptions {
            contrast_threshold: 2.0,
            ..CheckOptions::default()
        };
        let page = page_from_html(r#"<p style="color: #999999">Body text</p>"#);
        assert!(check_contrast_minimum(&page, &options).is_empty());
    }
}
