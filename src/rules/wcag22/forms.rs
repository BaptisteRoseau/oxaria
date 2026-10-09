//! WCAG 1.3.1 / 3.3.2 / 4.1.2 checks for form controls.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

/// H44: every labelable control needs a `label`, an ARIA name, or a wrapping `label`.
pub fn check_missing_label(page: &RenderedPage) -> Vec<Finding> {
    page.all()
        .filter(|el| is_labelable_control(*el))
        .filter(|control| control.accessible_name().is_empty())
        .map(|control| {
            Finding::error("H44", missing_label_message(control))
                .at(control)
                .help(missing_label_help(page, control))
        })
        .collect()
}

fn is_labelable_control(el: ElementRef) -> bool {
    matches!(el.tag(), "input" | "textarea" | "select") && is_labelable(el)
}

fn is_labelable(control: ElementRef) -> bool {
    let unlabelable_types = ["hidden", "submit", "button", "image", "reset"];
    match control.attr("type") {
        Some(input_type) => !unlabelable_types.contains(&input_type),
        None => true,
    }
}

fn missing_label_message(control: ElementRef) -> String {
    let name = control.tag();
    match control.attr("type") {
        Some(type_attr) => format!("<{name} type=\"{type_attr}\"> has no associated label"),
        None => format!("<{name}> has no associated label"),
    }
}

fn missing_label_help(page: &RenderedPage, control: ElementRef) -> String {
    let label = match control.attr("id").filter(|id| is_unique_id(page, id)) {
        Some(id) => format!("add <label for=\"{id}\">...</label>"),
        None => "wrap it in <label>...</label>".to_string(),
    };
    format!("{label}, or give it an aria-label (a placeholder is not a label)")
}

/// A `for=` pointing at a duplicate id would label the first element with it, not this one.
fn is_unique_id(page: &RenderedPage, id: &str) -> bool {
    page.all().filter(|el| el.attr("id") == Some(id)).count() == 1
}

/// F68: interactive controls (buttons, custom `role=button` widgets, image/submit buttons) must
/// expose an accessible name.
pub fn check_unnamed_control(page: &RenderedPage) -> Vec<Finding> {
    page.all()
        .filter(|el| is_unnamed_control_candidate(*el))
        .filter(|control| control.accessible_name().is_empty())
        .map(|control| {
            Finding::error("F68", unnamed_control_message(control))
                .at(control)
                .help(unnamed_control_help(control))
        })
        .collect()
}

fn is_unnamed_control_candidate(el: ElementRef) -> bool {
    el.tag() == "button"
        || el.attr("role") == Some("button")
        || (el.tag() == "input" && matches!(el.attr("type"), Some("submit" | "button" | "image")))
}

fn unnamed_control_message(control: ElementRef) -> String {
    format!(
        "<{}> has no accessible name (no text, aria-label, or aria-labelledby)",
        control.tag()
    )
}

fn unnamed_control_help(control: ElementRef) -> &'static str {
    match control.tag() {
        "button" => {
            "give it visible text, or aria-label=\"...\" if it only shows an icon \
             (and mark the icon aria-hidden=\"true\")"
        }
        "input" => "give it a value=\"...\" (or alt=\"...\" for type=\"image\") naming its action",
        _ => "use a native <button> with visible text, or add aria-label=\"...\"",
    }
}

/// H90: a `required` control's accessible name or description must tell assistive technology
/// users the field is required, not rely on visual styling alone.
pub fn check_required_not_indicated(page: &RenderedPage) -> Vec<Finding> {
    page.all()
        .filter(|el| el.has_attr("required"))
        .filter(|control| !required_is_indicated(page, *control))
        .map(|control| {
            Finding::error("H90", required_not_indicated_message(control))
                .at(control)
                .help(REQUIRED_NOT_INDICATED_HELP)
        })
        .collect()
}

const REQUIRED_NOT_INDICATED_HELP: &str = "say it in the label, e.g. \
    <label for=\"...\">Last name (required)</label>, or add aria-required=\"true\"";

fn required_is_indicated(page: &RenderedPage, control: ElementRef) -> bool {
    if control.attr("aria-required") == Some("true") {
        return true;
    }
    let name = control.accessible_name();
    let description = description_text(page, control);
    mentions_required(&name) || mentions_required(&description)
}

fn description_text(page: &RenderedPage, control: ElementRef) -> String {
    control
        .attr("aria-describedby")
        .map(|ids| page.ids_text(ids))
        .unwrap_or_default()
}

fn mentions_required(text: &str) -> bool {
    text.to_lowercase().contains("required")
}

fn required_not_indicated_message(control: ElementRef) -> String {
    format!(
        "<{}> is required but that isn't conveyed in its accessible name or description",
        control.tag()
    )
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::{help, rule_ids, run};

    #[test]
    fn missing_label_message_omits_absent_type_attribute() {
        // debian.org's search box, `<input name="P">`, was reported as `<input type="">`.
        let html = r#"<input name="P" value="" size="14">"#;
        let findings = run(check_missing_label, html);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "<input> has no associated label");
    }

    #[test]
    fn missing_label_help_uses_the_controls_id() {
        let html = r#"<input id="email" type="email"><textarea></textarea>"#;
        let findings = run(check_missing_label, html);
        let help = |index: usize| findings[index].help.clone().unwrap();
        assert!(
            help(0).starts_with(r#"add <label for="email">"#),
            "{}",
            help(0)
        );
        assert!(help(1).starts_with("wrap it in <label>"), "{}", help(1));
    }

    #[test]
    fn missing_label_help_avoids_a_duplicate_id() {
        let html = r#"<input type="checkbox" id="a"><input type="checkbox" id="a">"#;
        assert!(help(check_missing_label, html).starts_with("wrap it in <label>"));
    }

    #[test]
    fn unnamed_custom_button_help_suggests_a_native_button() {
        let help = help(check_unnamed_control, r#"<div role="button"></div>"#);
        assert!(help.starts_with("use a native <button>"), "{help}");
    }

    #[rstest]
    #[case(r#"<input id="email" type="email">"#, &["H44"])]
    #[case(r#"<label for="email">Email</label><input id="email" type="email">"#, &[])]
    #[case(r#"<label>Email <input type="email"></label>"#, &[])]
    #[case(r#"<input type="search" aria-label="Search">"#, &[])]
    #[case(r#"<input type="hidden" value="1">"#, &[])]
    fn missing_label(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_missing_label, html), expected, "{html}");
    }

    #[rstest]
    #[case("<button><svg></svg></button>", &["F68"])]
    #[case("<button>Submit</button>", &[])]
    #[case(r#"<button aria-label="Close"><svg></svg></button>"#, &[])]
    fn unnamed_control(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_unnamed_control, html), expected, "{html}");
    }

    #[rstest]
    #[case(r#"<label for="n">Last name</label><input id="n" required>"#, &["H90"])]
    #[case(r#"<label for="n">Last name (required)</label><input id="n" required>"#, &[])]
    #[case(
        r#"<label for="n">Last name</label><input id="n" required aria-required="true">"#,
        &[]
    )]
    fn required_not_indicated(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(
            rule_ids(check_required_not_indicated, html),
            expected,
            "{html}"
        );
    }
}
