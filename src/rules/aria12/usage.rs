//! Semantic choices the markup alone can judge: `term` on interactive
//! elements and `time` text (ARIA-USAGE002), and naming mechanisms
//! (ARIA-USAGE003). The other choices depend on purpose and behavior.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    aria_idrefs, first_valid_role, has_author_name, is_focusable, is_hidden,
};
use crate::rules::{CheckOptions, Finding};

use super::support::{has_value, quoted};
use super::time_string;
use super::tree::has_exposed_role;

/// ARIA-USAGE002: a `term` isn't interactive, and a `time`'s text is a
/// machine-readable date, time or duration.
pub fn check_role_purpose(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_hidden(*el))
        .filter_map(|el| match first_valid_role(el).map(|role| role.name) {
            Some("term") if is_focusable(el) => Some(interactive_term_finding(el)),
            Some("time") => invalid_time_finding(el),
            _ => None,
        })
        .collect()
}

fn interactive_term_finding(el: ElementRef) -> Finding {
    Finding::error(
        "ARIA-USAGE002",
        format!("role=\"term\" is set on an interactive <{}>", el.tag()),
    )
    .at(el)
    .help(format!(
        "remove role=\"term\" from the <{}>, and wrap the term's text in a <dfn> (or an element \
         with role=\"term\") instead",
        el.tag()
    ))
}

fn invalid_time_finding(el: ElementRef) -> Option<Finding> {
    let text = el.text();
    (!text.is_empty() && !time_string::is_valid(&text)).then(|| {
        Finding::error(
            "ARIA-USAGE002",
            format!("role=\"time\" holds \"{text}\", which is not a valid date or time"),
        )
        .at(el)
        .help(
            "limit the text to a date, time or duration such as \"2019-11-18\", \"09:54:39\" or \
             \"4h 18m 3s\", or use a native <time datetime=\"...\"> for human-readable text",
        )
    })
}

/// ARIA-USAGE003: a placeholder isn't a label, an alert dialog's message
/// is its description, and details are for everyone to see.
pub fn check_naming_mechanism(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_hidden(*el))
        .flat_map(|el| {
            [
                placeholder_as_label(el),
                alertdialog_without_description(el),
            ]
            .into_iter()
            .flatten()
            .chain(hidden_details(el))
        })
        .collect()
}

fn placeholder_as_label(el: ElementRef) -> Option<Finding> {
    let placeholder = el
        .attr("aria-placeholder")
        .filter(|_| has_value(el, "aria-placeholder"))?;
    (!has_author_name(el) && !has_label_element(el)).then(|| {
        Finding::error(
            "ARIA-USAGE003",
            format!(
                "{} is the only label of this <{}>",
                quoted("aria-placeholder", placeholder),
                el.tag()
            ),
        )
        .at(el)
        .help(
            "label the field with aria-labelledby pointing at visible label text (or \
             aria-label); a placeholder is a hint that disappears once the user types",
        )
    })
}

fn has_label_element(el: ElementRef) -> bool {
    let is_labelled_by_for = el.attr("id").is_some_and(|id| {
        el.page()
            .by_tag("label")
            .any(|label| label.attr("for") == Some(id))
    });
    is_labelled_by_for || el.ancestors().any(|ancestor| ancestor.tag() == "label")
}

fn alertdialog_without_description(el: ElementRef) -> Option<Finding> {
    let is_undescribed =
        has_exposed_role(el, &["alertdialog"]) && !has_value(el, "aria-describedby");
    is_undescribed.then(|| {
        Finding::error(
            "ARIA-USAGE003",
            "alertdialog has no aria-describedby referencing its message".to_string(),
        )
        .at(el)
        .help("give the alert message an id and reference it with aria-describedby=\"<id>\"")
    })
}

fn hidden_details(el: ElementRef) -> Vec<Finding> {
    aria_idrefs(el, "aria-details")
        .into_iter()
        .filter(|id| el.page().element_by_id(id).is_some_and(is_hidden))
        .map(|id| {
            Finding::error(
                "ARIA-USAGE003",
                format!("aria-details=\"{id}\" references hidden content"),
            )
            .at(el)
            .help(format!(
                "make #{id} visible to every user (remove hidden or aria-hidden), or reference \
                 it with aria-describedby if it is only a description"
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(check: fn(&RenderedPage, &CheckOptions) -> Vec<Finding>, html: &str) -> Vec<Finding> {
        check(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(r#"<a href="/glossary#api" role="term">API</a>"#)]
    #[case(r#"<span role="time">Sept 27</span>"#)]
    #[case(r#"<div role="time">yesterday at noon</div>"#)]
    fn misused_roles_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_role_purpose, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<span role="term">API</span>"#)]
    #[case(r#"<span role="time">2019-11-18</span>"#)]
    #[case(r#"<span role="time">4h 18m 3s</span>"#)]
    #[case(r#"<time datetime="2026-09-27">Sept 27</time>"#)]
    #[case(r#"<span role="time"></span>"#)]
    fn fitting_roles_are_not_flagged(#[case] html: &str) {
        assert!(run(check_role_purpose, html).is_empty(), "{html}");
    }

    #[rstest]
    #[case(r#"<div contenteditable role="searchbox" aria-placeholder="MM-DD-YYYY"></div>"#)]
    #[case(r#"<div role="alertdialog" aria-labelledby="t"><h2 id="t">Delete?</h2><p>Gone forever.</p><button>OK</button></div>"#)]
    #[case(r#"<button aria-details="d">Chart</button><div id="d" hidden>Details</div>"#)]
    fn wrong_naming_mechanisms_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_naming_mechanism, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<span id="label">Birthday:</span>
              <div contenteditable role="searchbox" aria-labelledby="label" aria-placeholder="MM-DD-YYYY"></div>"#)]
    #[case(r#"<label for="b">Birthday</label><div id="b" contenteditable role="textbox" aria-placeholder="MM-DD"></div>"#)]
    #[case(r#"<div role="alertdialog" aria-describedby="m"><p id="m">Gone forever.</p><button>OK</button></div>"#)]
    #[case(r#"<button aria-details="d">Chart</button><div id="d">Details</div>"#)]
    fn right_naming_mechanisms_are_not_flagged(#[case] html: &str) {
        assert!(run(check_naming_mechanism, html).is_empty(), "{html}");
    }
}
