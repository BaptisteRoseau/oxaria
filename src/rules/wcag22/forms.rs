//! WCAG 1.3.1 / 3.3.2 / 4.1.2 checks for form controls.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::{CheckOptions, Finding};

/// H44: every labelable control needs a `label`, an ARIA name, or a wrapping `label`.
pub fn check_missing_label(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
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
pub fn check_unnamed_control(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
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
pub fn check_required_not_indicated(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
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
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn missing_label_message_omits_absent_type_attribute() {
        // debian.org's search box, `<input name="P">`, was reported as `<input type="">`.
        let p = page_from_html(r#"<input name="P" value="" size="14">"#);
        let findings = check_missing_label(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "<input> has no associated label");
    }

    #[test]
    fn missing_label_help_uses_the_controls_id() {
        let p = page_from_html(r#"<input id="email" type="email"><textarea></textarea>"#);
        let findings = check_missing_label(&p, &CheckOptions::default());
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
        let p = page_from_html(r#"<input type="checkbox" id="a"><input type="checkbox" id="a">"#);
        let findings = check_missing_label(&p, &CheckOptions::default());
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with("wrap it in <label>")
        );
    }

    #[test]
    fn unnamed_custom_button_help_suggests_a_native_button() {
        let p = page_from_html(r#"<div role="button"></div>"#);
        let help = check_unnamed_control(&p, &CheckOptions::default())[0]
            .help
            .clone()
            .unwrap();
        assert!(help.starts_with("use a native <button>"), "{help}");
    }

    #[test]
    fn input_without_label_is_flagged() {
        let p = page_from_html(r#"<input id="email" type="email">"#);
        let findings = check_missing_label(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H44");
    }

    #[test]
    fn input_with_matching_label_for_is_not_flagged() {
        let p =
            page_from_html(r#"<label for="email">Email</label><input id="email" type="email">"#);
        assert!(check_missing_label(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn input_wrapped_in_label_is_not_flagged() {
        let p = page_from_html(r#"<label>Email <input type="email"></label>"#);
        assert!(check_missing_label(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn input_with_aria_label_is_not_flagged() {
        let p = page_from_html(r#"<input type="search" aria-label="Search">"#);
        assert!(check_missing_label(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn hidden_input_is_not_flagged() {
        let p = page_from_html(r#"<input type="hidden" value="1">"#);
        assert!(check_missing_label(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn icon_button_without_name_is_flagged() {
        let p = page_from_html("<button><svg></svg></button>");
        let findings = check_unnamed_control(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F68");
    }

    #[test]
    fn button_with_text_is_not_flagged() {
        let p = page_from_html("<button>Submit</button>");
        assert!(check_unnamed_control(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn button_with_aria_label_is_not_flagged() {
        let p = page_from_html(r#"<button aria-label="Close"><svg></svg></button>"#);
        assert!(check_unnamed_control(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn required_field_without_indication_is_flagged() {
        let p = page_from_html(r#"<label for="n">Last name</label><input id="n" required>"#);
        let findings = check_required_not_indicated(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H90");
    }

    #[test]
    fn required_field_with_label_wording_is_not_flagged() {
        let p =
            page_from_html(r#"<label for="n">Last name (required)</label><input id="n" required>"#);
        assert!(check_required_not_indicated(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn required_field_with_aria_required_is_not_flagged() {
        let p = page_from_html(
            r#"<label for="n">Last name</label><input id="n" required aria-required="true">"#,
        );
        assert!(check_required_not_indicated(&p, &CheckOptions::default()).is_empty());
    }
}
