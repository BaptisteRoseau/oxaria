//! ARIA in HTML's requirements on `aria-*` attributes used alongside their
//! native HTML equivalents.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{AriaValue, NATIVE_EQUIVALENTS, Parsed, aria_value, input_type};
use crate::rules::{CheckOptions, Finding};

use super::markup::{attribute, start_tag};

/// `input` types whose `min`/`max` HTML defines.
const RANGED_INPUT_TYPES: &[&str] = &[
    "date",
    "datetime-local",
    "month",
    "number",
    "range",
    "time",
    "week",
];

/// A native attribute and its `aria-*` counterpart, with the condition that
/// makes their use together fail a rule.
struct Pairing {
    native: &'static str,
    aria: &'static str,
    fails: fn(ElementRef) -> bool,
    help: fn(ElementRef) -> String,
}

const CONFLICTS: &[Pairing] = &[
    Pairing {
        native: "checked",
        aria: "aria-checked",
        fails: is_checkable_with_aria_checked,
        help: |_| {
            "remove aria-checked: the checked attribute (or the indeterminate IDL attribute for \
             \"mixed\") already exposes the state"
                .to_string()
        },
    },
    Pairing {
        native: "disabled",
        aria: "aria-disabled",
        fails: |el| has_native_and_aria(el, "disabled", "aria-disabled", AriaValue::False),
        help: |_| "remove aria-disabled=\"false\": the element is disabled".to_string(),
    },
    Pairing {
        native: "required",
        aria: "aria-required",
        fails: |el| has_native_and_aria(el, "required", "aria-required", AriaValue::False),
        help: |_| "remove aria-required=\"false\": the element is required".to_string(),
    },
    Pairing {
        native: "readonly",
        aria: "aria-readonly",
        fails: |el| has_native_and_aria(el, "readonly", "aria-readonly", AriaValue::False),
        help: |_| "remove aria-readonly=\"false\": the element is read-only".to_string(),
    },
    Pairing {
        native: "contenteditable",
        aria: "aria-readonly",
        fails: |el| is_editable(el) && is_aria(el, "aria-readonly", AriaValue::True),
        help: |_| {
            "remove aria-readonly=\"true\", or set contenteditable=\"false\" if the content \
             shouldn't be editable"
                .to_string()
        },
    },
    Pairing {
        native: "placeholder",
        aria: "aria-placeholder",
        fails: |el| native_value(el, "placeholder").is_some() && el.has_attr("aria-placeholder"),
        help: |_| {
            "remove aria-placeholder: the placeholder attribute already provides the hint"
                .to_string()
        },
    },
    Pairing {
        native: "max",
        aria: "aria-valuemax",
        fails: |el| has_ranged(el, "max") && values_differ(el, "max", "aria-valuemax"),
        help: |el| keep_native_help(el, "max", "aria-valuemax"),
    },
    Pairing {
        native: "min",
        aria: "aria-valuemin",
        fails: |el| has_ranged(el, "min") && values_differ(el, "min", "aria-valuemin"),
        help: |el| keep_native_help(el, "min", "aria-valuemin"),
    },
    Pairing {
        native: "colspan",
        aria: "aria-colspan",
        fails: |el| values_differ(el, "colspan", "aria-colspan"),
        help: |el| keep_native_help(el, "colspan", "aria-colspan"),
    },
    Pairing {
        native: "rowspan",
        aria: "aria-rowspan",
        fails: |el| values_differ(el, "rowspan", "aria-rowspan"),
        help: |el| keep_native_help(el, "rowspan", "aria-rowspan"),
    },
];

const REPETITIONS: &[Pairing] = &[
    Pairing {
        native: "disabled",
        aria: "aria-disabled",
        fails: |el| has_native_and_aria(el, "disabled", "aria-disabled", AriaValue::True),
        help: |el| keep_native_help(el, "disabled", "aria-disabled"),
    },
    Pairing {
        native: "required",
        aria: "aria-required",
        fails: |el| has_native_and_aria(el, "required", "aria-required", AriaValue::True),
        help: |el| keep_native_help(el, "required", "aria-required"),
    },
    Pairing {
        native: "readonly",
        aria: "aria-readonly",
        fails: |el| has_native_and_aria(el, "readonly", "aria-readonly", AriaValue::True),
        help: |el| keep_native_help(el, "readonly", "aria-readonly"),
    },
    Pairing {
        native: "colspan",
        aria: "aria-colspan",
        fails: |el| values_match(el, "colspan", "aria-colspan"),
        help: |el| keep_native_help(el, "colspan", "aria-colspan"),
    },
    Pairing {
        native: "rowspan",
        aria: "aria-rowspan",
        fails: |el| values_match(el, "rowspan", "aria-rowspan"),
        help: |el| keep_native_help(el, "rowspan", "aria-rowspan"),
    },
    Pairing {
        native: "max",
        aria: "aria-valuemax",
        fails: |el| repeats_or_replaces(el, "max", "aria-valuemax"),
        help: |el| use_native_help(el, "max", "aria-valuemax"),
    },
    Pairing {
        native: "min",
        aria: "aria-valuemin",
        fails: |el| repeats_or_replaces(el, "min", "aria-valuemin"),
        help: |el| use_native_help(el, "min", "aria-valuemin"),
    },
];

/// HTMLARIA013: an `aria-*` attribute contradicting (or, for `checked`
/// and `placeholder`, standing in for) its native equivalent.
pub fn check_conflicting_aria(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    pairing_findings(page, CONFLICTS, "HTMLARIA013", "conflicts with")
}

/// HTMLARIA014: an `aria-*` attribute repeating its native equivalent, or
/// `aria-valuemin`/`aria-valuemax` on an element with native `min`/`max`.
pub fn check_repeated_aria(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    pairing_findings(page, REPETITIONS, "HTMLARIA014", "duplicates")
}

fn pairing_findings(
    page: &RenderedPage,
    pairings: &'static [Pairing],
    rule_id: &'static str,
    verb: &'static str,
) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            pairings
                .iter()
                .filter(move |pairing| (pairing.fails)(el))
                .map(move |pairing| {
                    Finding::error(rule_id, pairing_message(el, pairing, verb))
                        .at(el)
                        .help((pairing.help)(el))
                })
        })
        .collect()
}

fn pairing_message(el: ElementRef, pairing: &Pairing, verb: &str) -> String {
    format!(
        "{}: {} {verb} the native {} attribute",
        start_tag(el, &["type", pairing.native, pairing.aria]),
        pairing.aria,
        pairing.native
    )
}

fn keep_native_help(el: ElementRef, native: &str, aria: &str) -> String {
    let native = attribute(el, native).unwrap_or_else(|| native.to_string());
    format!("remove {aria} and keep {native}")
}

fn use_native_help(el: ElementRef, native: &str, aria: &str) -> String {
    match (el.attr(native), el.attr(aria)) {
        (Some(_), _) => keep_native_help(el, native, aria),
        (None, Some(value)) => format!("replace {aria}=\"{value}\" with {native}=\"{value}\""),
        (None, None) => format!("remove {aria}"),
    }
}

/// The native attribute's value, on the elements it applies to.
fn native_value<'a>(el: ElementRef<'a>, native: &str) -> Option<&'a str> {
    let applies = NATIVE_EQUIVALENTS.iter().any(|equivalent| {
        equivalent.native == native
            && equivalent
                .elements
                .is_none_or(|tags| tags.contains(&el.tag()))
    });
    applies.then(|| el.attr(native)).flatten()
}

fn is_aria(el: ElementRef, aria: &str, expected: AriaValue) -> bool {
    aria_value(el, aria) == Some(Parsed::Valid(expected))
}

fn has_native_and_aria(el: ElementRef, native: &str, aria: &str, expected: AriaValue) -> bool {
    native_value(el, native).is_some() && is_aria(el, aria, expected)
}

fn is_checkable_with_aria_checked(el: ElementRef) -> bool {
    el.tag() == "input"
        && matches!(input_type(el), "checkbox" | "radio")
        && el.has_attr("aria-checked")
}

/// An editing host, or inside one: the nearest `contenteditable` decides.
fn is_editable(el: ElementRef) -> bool {
    std::iter::successors(Some(el), |el| el.parent())
        .find_map(|el| el.attr("contenteditable"))
        .is_some_and(|value| {
            let value = value.trim();
            value.is_empty()
                || value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("plaintext-only")
        })
}

/// Elements where HTML defines `min`/`max` (`progress` has no `min`).
fn has_ranged(el: ElementRef, native: &str) -> bool {
    match el.tag() {
        "meter" => true,
        "progress" => native == "max",
        "input" => RANGED_INPUT_TYPES.contains(&input_type(el)),
        _ => false,
    }
}

fn repeats_or_replaces(el: ElementRef, native: &str, aria: &str) -> bool {
    has_ranged(el, native)
        && el.has_attr(aria)
        && (!el.has_attr(native) || values_match(el, native, aria))
}

fn values_differ(el: ElementRef, native: &str, aria: &str) -> bool {
    native_value(el, native).is_some() && el.has_attr(aria) && !values_match(el, native, aria)
}

fn values_match(el: ElementRef, native: &str, aria: &str) -> bool {
    match (native_value(el, native), el.attr(aria)) {
        (Some(native), Some(aria)) => same_number(native, aria),
        _ => false,
    }
}

fn same_number(a: &str, b: &str) -> bool {
    let (a, b) = (a.trim(), b.trim());
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
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
    #[case(r#"<input type="checkbox" checked aria-checked="false">"#)]
    #[case(r#"<input type="radio" aria-checked="true">"#)]
    #[case(r#"<input type="text" required aria-required="false">"#)]
    #[case(r#"<input type="text" placeholder="Search" aria-placeholder="Search">"#)]
    #[case(r#"<select disabled aria-disabled="false"><option>a</option></select>"#)]
    #[case(r#"<select><option disabled aria-disabled="false">a</option></select>"#)]
    #[case(r#"<textarea readonly aria-readonly="false"></textarea>"#)]
    #[case(r#"<div contenteditable aria-readonly="true">x</div>"#)]
    #[case(r#"<div contenteditable="true"><p aria-readonly="true">x</p></div>"#)]
    #[case(r#"<meter value="3" max="10" aria-valuemax="5"></meter>"#)]
    #[case(r#"<input type="range" min="0" aria-valuemin="1">"#)]
    #[case(r#"<table><tr><td colspan="2" aria-colspan="3">x</td></tr></table>"#)]
    #[case(r#"<table><tr><th rowspan="2" aria-rowspan="1">x</th></tr></table>"#)]
    fn conflicting_attributes_are_flagged(#[case] html: &str) {
        let findings = run(check_conflicting_aria, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA013");
    }

    #[rstest]
    #[case(r#"<input type="checkbox" checked>"#)]
    #[case(r#"<input type="text" required>"#)]
    #[case(r#"<input type="text" placeholder="Search">"#)]
    #[case(r#"<div role="checkbox" aria-checked="true">x</div>"#)]
    #[case(r#"<button type="button" aria-disabled="false">x</button>"#)]
    #[case(r#"<div contenteditable="false"><p aria-readonly="true">x</p></div>"#)]
    #[case(r#"<div aria-placeholder="x" placeholder="y">x</div>"#)]
    #[case(r#"<meter value="3" max="10" aria-valuemax="10"></meter>"#)]
    #[case(r#"<table><tr><td colspan="2" aria-colspan="2">x</td></tr></table>"#)]
    #[case(r#"<div role="cell" colspan="2" aria-colspan="3">x</div>"#)]
    fn consistent_attributes_do_not_conflict(#[case] html: &str) {
        assert!(run(check_conflicting_aria, html).is_empty());
    }

    #[test]
    fn conflict_message_quotes_the_markup() {
        let findings = run(
            check_conflicting_aria,
            r#"<input type="checkbox" checked aria-checked="false">"#,
        );
        assert_eq!(
            findings[0].message,
            r#"<input type="checkbox" checked aria-checked="false">: aria-checked conflicts with the native checked attribute"#
        );
    }

    #[test]
    fn differing_span_help_keeps_the_native_value() {
        let findings = run(
            check_conflicting_aria,
            r#"<table><tr><td colspan="2" aria-colspan="3">x</td></tr></table>"#,
        );
        assert_eq!(
            findings[0].help.as_deref(),
            Some(r#"remove aria-colspan and keep colspan="2""#)
        );
    }

    #[rstest]
    #[case(r#"<button disabled aria-disabled="true">Save</button>"#)]
    #[case(r#"<progress value="30" max="100" aria-valuemax="100"></progress>"#)]
    #[case(r#"<input required aria-required="true">"#)]
    #[case(r#"<input readonly aria-readonly="TRUE">"#)]
    #[case(r#"<meter value="3" aria-valuemin="0"></meter>"#)]
    #[case(r#"<input type="number" aria-valuemax="9">"#)]
    #[case(r#"<table><tr><td rowspan="2" aria-rowspan="2">x</td></tr></table>"#)]
    fn repeated_attributes_are_flagged(#[case] html: &str) {
        let findings = run(check_repeated_aria, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA014");
    }

    #[rstest]
    #[case("<button disabled>Save</button>")]
    #[case(r#"<progress value="30" max="100"></progress>"#)]
    #[case(r#"<progress value="30" aria-valuemin="0"></progress>"#)]
    #[case(r#"<input role="spinbutton" aria-valuemax="9" aria-valuenow="1">"#)]
    #[case(r#"<div role="slider" aria-valuemax="9" aria-valuenow="1">x</div>"#)]
    #[case(r#"<meter value="3" max="10" aria-valuemax="5"></meter>"#)]
    #[case(r#"<div aria-disabled="true" disabled>x</div>"#)]
    fn attributes_that_do_not_repeat_are_not_flagged(#[case] html: &str) {
        assert!(run(check_repeated_aria, html).is_empty());
    }

    #[test]
    fn missing_native_limit_help_suggests_the_native_attribute() {
        let findings = run(
            check_repeated_aria,
            r#"<input type="number" aria-valuemax="9">"#,
        );
        assert_eq!(
            findings[0].help.as_deref(),
            Some(r#"replace aria-valuemax="9" with max="9""#)
        );
    }
}
