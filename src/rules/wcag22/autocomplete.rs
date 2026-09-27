//! WCAG 1.3.5 (Identify Input Purpose) check for `autocomplete` values.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::{CheckOptions, Finding};

/// The WCAG 2.2 input purposes (`guidelines/input-purposes.html`) that aren't contact
/// details, plus `one-time-code`, which the HTML autofill list added later.
const FIELD_NAMES: &[&str] = &[
    "name",
    "honorific-prefix",
    "given-name",
    "additional-name",
    "family-name",
    "honorific-suffix",
    "nickname",
    "organization-title",
    "username",
    "new-password",
    "current-password",
    "one-time-code",
    "organization",
    "street-address",
    "address-line1",
    "address-line2",
    "address-line3",
    "address-level4",
    "address-level3",
    "address-level2",
    "address-level1",
    "country",
    "country-name",
    "postal-code",
    "cc-name",
    "cc-given-name",
    "cc-additional-name",
    "cc-family-name",
    "cc-number",
    "cc-exp",
    "cc-exp-month",
    "cc-exp-year",
    "cc-csc",
    "cc-type",
    "transaction-currency",
    "transaction-amount",
    "language",
    "bday",
    "bday-day",
    "bday-month",
    "bday-year",
    "sex",
    "url",
    "photo",
];

/// The only purposes a `home`/`work`/`mobile`/`fax`/`pager` qualifier may precede.
const CONTACT_FIELD_NAMES: &[&str] = &[
    "tel",
    "tel-country-code",
    "tel-national",
    "tel-area-code",
    "tel-local",
    "tel-local-prefix",
    "tel-local-suffix",
    "tel-extension",
    "email",
    "impp",
];

const CONTACT_QUALIFIERS: &[&str] = &["home", "work", "mobile", "fax", "pager"];

/// F107: an `autocomplete` value outside the HTML autofill grammar identifies no input
/// purpose, so neither browsers nor assistive technology can act on it.
pub fn check_invalid_autocomplete(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_autofillable(*el))
        .filter_map(|control| Some((control, control.attr("autocomplete")?)))
        .filter(|(_, value)| !is_valid_autocomplete(value))
        .map(|(control, value)| {
            Finding::error(
                "F107",
                format!(
                    "<{}> has an invalid autocomplete value \"{value}\"",
                    control.tag()
                ),
            )
            .at(control)
            .help(
                "use one of the input purposes from WCAG's list, e.g. autocomplete=\"email\" \
                 or \"bday\" (not \"birthday\"), optionally preceded by shipping/billing",
            )
        })
        .collect()
}

fn is_autofillable(el: ElementRef) -> bool {
    match el.tag() {
        "input" => el.attr("type") != Some("hidden"),
        "select" | "textarea" => true,
        _ => false,
    }
}

/// `[section-*] [shipping|billing] [home|work|mobile|fax|pager] <field> [webauthn]`, or a
/// lone `on`/`off`, per the HTML autofill grammar.
pub(crate) fn is_valid_autocomplete(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let mut tokens: Vec<&str> = lower.split_ascii_whitespace().collect();
    if matches!(tokens.as_slice(), [] | ["on"] | ["off"]) {
        return true;
    }
    if tokens.last() == Some(&"webauthn") {
        tokens.pop();
    }
    let Some(field) = tokens.pop() else {
        return false;
    };
    let qualified = tokens
        .last()
        .is_some_and(|token| CONTACT_QUALIFIERS.contains(token));
    if qualified {
        tokens.pop();
    }
    let field_is_valid = match qualified {
        true => CONTACT_FIELD_NAMES.contains(&field),
        false => FIELD_NAMES.contains(&field) || CONTACT_FIELD_NAMES.contains(&field),
    };
    field_is_valid && has_valid_scope(&tokens)
}

fn has_valid_scope(tokens: &[&str]) -> bool {
    let is_section = |token: &&str| token.starts_with("section-");
    let is_address_type = |token: &&str| matches!(*token, "shipping" | "billing");
    match tokens {
        [] => true,
        [only] => is_section(only) || is_address_type(only),
        [section, address_type] => is_section(section) && is_address_type(address_type),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn made_up_value_is_flagged() {
        let p = page_from_html(
            r#"<label for="b">Birthday:</label><input autocomplete="birthday" id="b" type="text">"#,
        );
        let findings = check_invalid_autocomplete(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F107");
        assert!(findings[0].message.contains("\"birthday\""));
    }

    #[test]
    fn hidden_inputs_and_other_elements_are_ignored() {
        let p = page_from_html(
            r#"<input type="hidden" autocomplete="nope"><form autocomplete="nope"></form>"#,
        );
        assert!(check_invalid_autocomplete(&p, &CheckOptions::default()).is_empty());
    }

    #[rstest]
    #[case("email")]
    #[case("bday")]
    #[case("off")]
    #[case("One-Time-Code")]
    #[case("billing street-address")]
    #[case("section-blue shipping postal-code")]
    #[case("work email")]
    #[case("username webauthn")]
    fn valid_values(#[case] value: &str) {
        assert!(is_valid_autocomplete(value), "{value}");
    }

    #[rstest]
    #[case("birthday")]
    #[case("nope")]
    #[case("home name")]
    #[case("shipping section-a name")]
    #[case("email on")]
    #[case("webauthn")]
    fn invalid_values(#[case] value: &str) {
        assert!(!is_valid_autocomplete(value), "{value}");
    }
}
