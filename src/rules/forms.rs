//! WCAG 1.3.1 / 3.3.2 / 4.1.2 checks for form controls.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

/// H44: every labelable control needs a `label`, an ARIA name, or a wrapping `label`.
pub fn check_missing_label(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_labelable_control(*el))
        .filter(|control| control.accessible_name().is_empty())
        .map(|control| Finding::error("H44", missing_label_message(control)).at(control))
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

/// F68: interactive controls (buttons, custom `role=button` widgets, image/submit buttons) must
/// expose an accessible name.
pub fn check_unnamed_control(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_unnamed_control_candidate(*el))
        .filter(|control| control.accessible_name().is_empty())
        .map(|control| Finding::error("F68", unnamed_control_message(control)).at(control))
        .collect()
}

fn is_unnamed_control_candidate(el: ElementRef) -> bool {
    el.tag() == "button"
        || el.attr("role") == Some("button")
        || (el.tag() == "input"
            && matches!(
                el.attr("type"),
                Some("submit") | Some("button") | Some("image")
            ))
}

fn unnamed_control_message(control: ElementRef) -> String {
    format!(
        "<{}> has no accessible name (no text, aria-label, or aria-labelledby)",
        control.tag()
    )
}

/// H90: a `required` control's accessible name or description must tell assistive technology
/// users the field is required, not rely on visual styling alone.
pub fn check_required_not_indicated(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| el.has_attr("required"))
        .filter(|control| !required_is_indicated(page, *control))
        .map(|control| Finding::error("H90", required_not_indicated_message(control)).at(control))
        .collect()
}

fn required_is_indicated(page: &RenderedPage, control: ElementRef) -> bool {
    if control.attr("aria-required") == Some("true") {
        return true;
    }
    let name = control.accessible_name();
    let description = description_text(page, control);
    mentions_required(&name) || mentions_required(&description)
}

fn description_text(page: &RenderedPage, control: ElementRef) -> String {
    match control.attr("aria-describedby") {
        Some(ids) => page.ids_text(ids),
        None => String::new(),
    }
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
