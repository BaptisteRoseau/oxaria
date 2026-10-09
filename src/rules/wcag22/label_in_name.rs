//! WCAG 2.5.3 (Label in Name) check for controls named by ARIA.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

const WIDGET_ROLES: &[&str] = &[
    "button", "link", "menuitem", "tab", "checkbox", "radio", "switch", "option",
];

/// F96: when `aria-label`/`aria-labelledby` overrides a control's visible label, the
/// resulting name must still contain that label, or speech users can't activate the control by
/// saying what they see.
pub fn check_label_not_in_name(page: &RenderedPage) -> Vec<Finding> {
    page.all()
        .filter(|el| is_control(*el) && is_named_by_aria(*el))
        .filter_map(|control| Some((control, visible_label(control)?)))
        .filter(|(control, label)| !contains_label(&control.accessible_name(), label))
        .map(|(control, label)| {
            let name = control.accessible_name();
            Finding::error_at(
                control,
                "F96",
                format!(
                    "<{}> shows \"{label}\" but its accessible name is \"{name}\"",
                    control.tag()
                ),
            )
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
        .any(|attr| el.non_empty_attr(attr).is_some())
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
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::{assert_fires, count};

    #[test]
    fn aria_label_replacing_button_text_is_flagged() {
        let finding = assert_fires(
            check_label_not_in_name,
            r#"<button id="sitesearch" aria-label="Find in this site">Go</button>"#,
            "F96",
        );
        assert_eq!(
            finding.message,
            "<button> shows \"Go\" but its accessible name is \"Find in this site\""
        );
    }

    #[rstest]
    // aria-labelledby replacing a submit button's value.
    #[case(
        r#"<div id="hidden-label">Find in this site</div>
           <input type="submit" aria-labelledby="hidden-label" value="search">"#,
        1
    )]
    // aria-label replacing a form label.
    #[case(
        r#"<label for="e">Email</label><input id="e" aria-label="Your address">"#,
        1
    )]
    // A name extending the visible label.
    #[case(
        r#"<a href="/spec" aria-label="Download the gizmo specification (PDF)">Download</a>
           <button aria-label="Send message now">Send message</button>"#,
        0
    )]
    // Case and punctuation are ignored.
    #[case(r#"<button aria-label="next page">Next page ›</button>"#, 0)]
    // Symbols and single characters are not labels.
    #[case(
        r#"<button aria-label="Close dialog">×</button><button aria-label="Close">x</button>
           <button aria-label="Bold">B</button>"#,
        0
    )]
    // Controls without ARIA names.
    #[case(r#"<button>Go</button><div aria-label="Card">Text</div>"#, 0)]
    // The label's words must appear in order.
    #[case(r#"<button aria-label="Cart view">View cart</button>"#, 1)]
    fn label_not_in_name(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_label_not_in_name, html), expected, "{html}");
    }
}
