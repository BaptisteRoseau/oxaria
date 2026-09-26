//! WCAG 2.5.3 (Label in Name) check for controls named by ARIA.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

const WIDGET_ROLES: &[&str] = &[
    "button", "link", "menuitem", "tab", "checkbox", "radio", "switch", "option",
];

/// F96: when `aria-label`/`aria-labelledby` overrides a control's visible label, the
/// resulting name must still contain that label, or speech users can't activate the control by
/// saying what they see.
pub fn check_label_not_in_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_control(*el) && is_named_by_aria(*el))
        .filter_map(|control| Some((control, visible_label(control)?)))
        .filter(|(control, label)| !contains_label(&control.accessible_name(), label))
        .map(|(control, label)| {
            let name = control.accessible_name();
            Finding::error(
                "F96",
                format!(
                    "<{}> shows \"{label}\" but its accessible name is \"{name}\"",
                    control.tag()
                ),
            )
            .at(control)
            .help(format!(
                "start the aria-label with the visible text, e.g. aria-label=\"{label} ...\", \
                 or drop it so the visible text is the name"
            ))
        })
        .collect()
}

fn is_control(el: ElementRef) -> bool {
    match el.tag() {
        "button" | "select" | "textarea" | "input" => el.attr("type") != Some("hidden"),
        "a" => el.has_attr("href"),
        _ => el
            .attr("role")
            .is_some_and(|role| WIDGET_ROLES.contains(&role)),
    }
}

fn is_named_by_aria(el: ElementRef) -> bool {
    ["aria-label", "aria-labelledby"]
        .iter()
        .any(|attr| el.attr(attr).is_some_and(|value| !value.trim().is_empty()))
}

/// Symbols ("×", "›") and single characters ("x" for close, "B" for bold) are not labels the
/// user would speak, per the Understanding document for 2.5.3.
fn visible_label(control: ElementRef) -> Option<String> {
    let label = match (control.tag(), control.attr("type")) {
        ("input", Some("submit" | "button" | "reset")) => control.attr("value")?.to_string(),
        ("input" | "select" | "textarea", _) => control.label_text()?,
        _ => control.text(),
    };
    let label = label.trim().to_string();
    let is_text = words(&label).iter().any(|word| word.chars().count() > 1);
    is_text.then_some(label)
}

/// Case and punctuation are ignored, as the Understanding document for 2.5.3 allows.
fn contains_label(name: &str, label: &str) -> bool {
    let name = words(name);
    let label = words(label);
    name.windows(label.len())
        .any(|window| window == label.as_slice())
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn findings(html: &str) -> Vec<Finding> {
        check_label_not_in_name(&page_from_html(html), &CheckOptions::default())
    }

    #[test]
    fn aria_label_replacing_button_text_is_flagged() {
        let findings =
            findings(r#"<button id="sitesearch" aria-label="Find in this site">Go</button>"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F96");
        assert_eq!(
            findings[0].message,
            "<button> shows \"Go\" but its accessible name is \"Find in this site\""
        );
    }

    #[test]
    fn labelledby_replacing_submit_value_is_flagged() {
        let html = r#"<div id="hidden-label">Find in this site</div>
                      <input type="submit" aria-labelledby="hidden-label" value="search">"#;
        assert_eq!(findings(html).len(), 1);
    }

    #[test]
    fn aria_label_replacing_a_form_label_is_flagged() {
        let html = r#"<label for="e">Email</label><input id="e" aria-label="Your address">"#;
        assert_eq!(findings(html).len(), 1);
    }

    #[test]
    fn name_extending_the_visible_label_is_not_flagged() {
        let html = r#"<a href="/spec" aria-label="Download the gizmo specification (PDF)">Download</a>
                      <button aria-label="Send message now">Send message</button>"#;
        assert!(findings(html).is_empty());
    }

    #[test]
    fn case_and_punctuation_are_ignored() {
        assert!(findings(r#"<button aria-label="next page">Next page ›</button>"#).is_empty());
    }

    #[test]
    fn symbol_labels_are_not_flagged() {
        let html = r#"<button aria-label="Close dialog">×</button><button aria-label="Close">x</button>
                      <button aria-label="Bold">B</button>"#;
        assert!(findings(html).is_empty());
    }

    #[test]
    fn controls_without_aria_names_are_not_flagged() {
        assert!(findings(r#"<button>Go</button><div aria-label="Card">Text</div>"#).is_empty());
    }

    #[test]
    fn words_must_appear_in_order() {
        assert_eq!(
            findings(r#"<button aria-label="Cart view">View cart</button>"#).len(),
            1
        );
    }
}
