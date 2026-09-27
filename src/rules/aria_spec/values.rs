//! Reading `aria-*` values by their WAI-ARIA 1.2 value type. Surrounding
//! whitespace is ignored, and literal values and tokens match ASCII
//! case-insensitively, as ARIA in HTML's case-sensitivity section asks of
//! user agents.

use crate::page::ElementRef;

use super::attributes::{Attribute, ValueType, attribute};

#[derive(Debug, Clone, PartialEq)]
pub enum AriaValue<'a> {
    True,
    False,
    Mixed,
    Undefined,
    Integer(i64),
    Number(f64),
    /// One id for an ID reference, any number for an ID reference list.
    IdRefs(Vec<&'a str>),
    String(&'a str),
    /// One token for a token type, any number for a token list, as written.
    Tokens(Vec<&'a str>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Parsed<'a> {
    /// Empty or only whitespace: WAI-ARIA treats it as absent.
    Empty,
    Valid(AriaValue<'a>),
    Invalid,
}

pub fn parse_value<'a>(attribute: &Attribute, raw: &'a str) -> Parsed<'a> {
    let value = raw.trim_matches(|c: char| c.is_ascii_whitespace());
    if value.is_empty() {
        return Parsed::Empty;
    }
    let parsed = match attribute.value_type {
        ValueType::TrueFalse | ValueType::TrueFalseUndefined | ValueType::Tristate => {
            literal(attribute, value)
        }
        ValueType::Integer => integer(value).map(AriaValue::Integer),
        ValueType::Number => number(value).map(AriaValue::Number),
        ValueType::IdRef => single(value).map(|id| AriaValue::IdRefs(vec![id])),
        ValueType::IdRefList => Some(AriaValue::IdRefs(value.split_ascii_whitespace().collect())),
        ValueType::String => Some(AriaValue::String(value)),
        ValueType::Token => single(value)
            .filter(|token| is_token_of(attribute, token))
            .map(|token| AriaValue::Tokens(vec![token])),
        ValueType::TokenList => token_list(attribute, value).map(AriaValue::Tokens),
    };
    parsed.map_or(Parsed::Invalid, Parsed::Valid)
}

/// `None` when the element doesn't have the attribute, or when it isn't a
/// WAI-ARIA 1.2 state or property.
pub fn aria_value<'a>(el: ElementRef<'a>, name: &str) -> Option<Parsed<'a>> {
    Some(parse_value(attribute(name)?, el.attr(name)?))
}

pub fn is_aria_true(el: ElementRef, name: &str) -> bool {
    aria_value(el, name) == Some(Parsed::Valid(AriaValue::True))
}

/// The ids an ID reference (list) attribute points at; empty when absent
/// or invalid.
pub fn aria_idrefs<'a>(el: ElementRef<'a>, name: &str) -> Vec<&'a str> {
    match aria_value(el, name) {
        Some(Parsed::Valid(AriaValue::IdRefs(ids))) => ids,
        _ => Vec::new(),
    }
}

pub fn aria_number(el: ElementRef, name: &str) -> Option<f64> {
    match aria_value(el, name)? {
        Parsed::Valid(AriaValue::Number(number)) => Some(number),
        Parsed::Valid(AriaValue::Integer(integer)) => Some(integer as f64),
        _ => None,
    }
}

fn literal<'a>(attribute: &Attribute, value: &str) -> Option<AriaValue<'a>> {
    let literal = attribute
        .values
        .iter()
        .find(|allowed| allowed.eq_ignore_ascii_case(value))?;
    match *literal {
        "true" => Some(AriaValue::True),
        "false" => Some(AriaValue::False),
        "mixed" => Some(AriaValue::Mixed),
        "undefined" => Some(AriaValue::Undefined),
        _ => None,
    }
}

fn single(value: &str) -> Option<&str> {
    (!value.contains(|c: char| c.is_ascii_whitespace())).then_some(value)
}

fn is_token_of(attribute: &Attribute, token: &str) -> bool {
    attribute
        .values
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(token))
}

fn token_list<'a>(attribute: &Attribute, value: &'a str) -> Option<Vec<&'a str>> {
    let tokens: Vec<&str> = value.split_ascii_whitespace().collect();
    tokens
        .iter()
        .all(|token| is_token_of(attribute, token))
        .then_some(tokens)
}

/// HTML's "valid integer": an optional `-` and ASCII digits.
fn integer(value: &str) -> Option<i64> {
    let digits = value.strip_prefix('-').unwrap_or(value);
    let is_valid = !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit());
    is_valid.then(|| value.parse().ok()).flatten()
}

/// HTML's "valid floating-point number": no `+`, no `inf`/`NaN`, digits on
/// both sides of the `.`.
fn number(value: &str) -> Option<f64> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (unsigned, None),
    };
    let is_digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    let mantissa_is_valid = match mantissa.split_once('.') {
        Some((whole, fraction)) => (whole.is_empty() || is_digits(whole)) && is_digits(fraction),
        None => is_digits(mantissa),
    };
    let exponent_is_valid =
        exponent.is_none_or(|e| is_digits(e.strip_prefix(['-', '+']).unwrap_or(e)));
    (mantissa_is_valid && exponent_is_valid)
        .then(|| value.parse().ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn valid(value: AriaValue<'static>) -> Parsed<'static> {
        Parsed::Valid(value)
    }

    #[rstest]
    #[case("aria-expanded", "true", valid(AriaValue::True))]
    #[case("aria-expanded", " FALSE ", valid(AriaValue::False))]
    #[case("aria-expanded", "undefined", valid(AriaValue::Undefined))]
    #[case("aria-expanded", "yes", Parsed::Invalid)]
    #[case("aria-expanded", "  ", Parsed::Empty)]
    #[case("aria-busy", "undefined", Parsed::Invalid)]
    #[case("aria-checked", "mixed", valid(AriaValue::Mixed))]
    #[case("aria-selected", "mixed", Parsed::Invalid)]
    #[case("aria-level", "2", valid(AriaValue::Integer(2)))]
    #[case("aria-level", "-1", valid(AriaValue::Integer(-1)))]
    #[case("aria-level", "2.5", Parsed::Invalid)]
    #[case("aria-level", "+2", Parsed::Invalid)]
    #[case("aria-valuenow", "2.5", valid(AriaValue::Number(2.5)))]
    #[case("aria-valuenow", "-.5e3", valid(AriaValue::Number(-500.0)))]
    #[case("aria-valuenow", "inf", Parsed::Invalid)]
    #[case("aria-valuenow", "5.", Parsed::Invalid)]
    #[case("aria-activedescendant", "opt1", valid(AriaValue::IdRefs(vec!["opt1"])))]
    #[case("aria-activedescendant", "opt1 opt2", Parsed::Invalid)]
    #[case("aria-controls", "a  b", valid(AriaValue::IdRefs(vec!["a", "b"])))]
    #[case("aria-label", " Close ", valid(AriaValue::String("Close")))]
    #[case("aria-live", "Polite", valid(AriaValue::Tokens(vec!["Polite"])))]
    #[case("aria-live", "rude", Parsed::Invalid)]
    #[case("aria-haspopup", "menu", valid(AriaValue::Tokens(vec!["menu"])))]
    #[case("aria-relevant", "additions text", valid(AriaValue::Tokens(vec!["additions", "text"])))]
    #[case("aria-relevant", "additions bogus", Parsed::Invalid)]
    fn parses_by_value_type(#[case] name: &str, #[case] raw: &str, #[case] expected: Parsed) {
        assert_eq!(parse_value(attribute(name).unwrap(), raw), expected);
    }

    #[test]
    fn reads_values_from_elements() {
        let page = page_from_html(
            r#"<div aria-hidden="true" aria-owns="a b" aria-valuenow="3" aria-bogus="true"></div>"#,
        );
        let div = page.by_tag("div").next().unwrap();
        assert!(is_aria_true(div, "aria-hidden"));
        assert!(!is_aria_true(div, "aria-busy"));
        assert_eq!(aria_idrefs(div, "aria-owns"), ["a", "b"]);
        assert_eq!(aria_number(div, "aria-valuenow"), Some(3.0));
        assert_eq!(aria_value(div, "aria-bogus"), None);
    }
}
