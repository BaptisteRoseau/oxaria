//! ACT rule 73f2c2 for `autocomplete` values.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    effective_role, input_type, is_aria_true, is_disabled, is_not_rendered, is_tabbable,
};
use crate::rules::wcag22::is_valid_autocomplete;
use crate::rules::{CheckOptions, Finding};

/// `input` types whose value the user doesn't type, so autofill doesn't
/// apply.
const FIXED_VALUE_TYPES: &[&str] = &[
    "button", "checkbox", "file", "image", "radio", "reset", "submit",
];

/// 73f2c2: an `autocomplete` value other than `on`/`off` must be a valid
/// autofill detail token list, on a field the user can fill in.
pub fn check_autocomplete_valid(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_fillable_field(*el))
        .filter_map(|field| Some((field, field.attr("autocomplete")?)))
        .filter(|(_, value)| !value.trim().is_empty() && !is_valid_autocomplete(value))
        .map(|(field, value)| {
            Finding::error(
                "73f2c2",
                format!(
                    "<{} autocomplete=\"{value}\"> is not a valid autofill token list",
                    field.tag()
                ),
            )
            .at(field)
            .help(
                "use space-separated tokens in this order: [section-*] [shipping|billing] \
                 [home|work|mobile|fax|pager, only before email/impp/tel*] <field name> \
                 [webauthn], e.g. autocomplete=\"shipping street-address\" or \"work email\"",
            )
        })
        .collect()
}

fn is_fillable_field(el: ElementRef) -> bool {
    let is_field = match el.tag() {
        "input" => !FIXED_VALUE_TYPES.contains(&input_type(el)),
        "select" | "textarea" => true,
        _ => false,
    };
    is_field
        && !is_disabled(el)
        && !is_aria_true(el, "aria-disabled")
        && !is_not_rendered(el)
        && !is_static(el)
}

/// Out of the tab order with a non-widget role, as `role="none"` makes it.
fn is_static(el: ElementRef) -> bool {
    !is_tabbable(el) && effective_role(el).is_none_or(|role| !role.is_a("widget"))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn count(html: &str) -> usize {
        check_autocomplete_valid(&page_from_html(html), &CheckOptions::default()).len()
    }

    #[rstest]
    #[case(r#"<label>Username<input autocomplete="badname"></label>"#)]
    #[case(r#"<label>Photo<input autocomplete="work photo"></label>"#)]
    #[case(r#"<label>Email<input autocomplete="work shipping email"></label>"#)]
    #[case(r#"<label>Email<input autocomplete="work,email"></label>"#)]
    #[case(r#"<label>Username<input role="banner" tabindex="0" autocomplete="banner"></label>"#)]
    #[case(r#"<label>Address<input autocomplete="shipping"></label>"#)]
    #[case(r#"<label>Address<input autocomplete="address-line1 address-line2"></label>"#)]
    #[case(r#"<label>Email<input autocomplete="work"></label>"#)]
    #[case(r#"<input type="password" autocomplete="current-password webauthn invalid">"#)]
    #[case(r#"<label>Email<input autocomplete="email invalid"></label>"#)]
    #[case(r#"<input autocomplete="badname" aria-hidden="true">"#)]
    fn invalid_values_fail(#[case] html: &str) {
        assert_eq!(count(html), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<label>Username<input autocomplete="username"></label>"#)]
    #[case(r#"<form autocomplete="off"><select autocomplete="bday-month"><option>Jan</option></select></form>"#)]
    #[case(r#"<label>Street<textarea autocomplete="Street-Address"></textarea></label>"#)]
    #[case(r#"<input autocomplete="work email">"#)]
    #[case(r#"<input autocomplete="section-partner email">"#)]
    #[case(r#"<input type="text" autocomplete="section-primary shipping work email">"#)]
    #[case(r#"<input type="password" autocomplete="current-password webauthn">"#)]
    #[case(r#"<input autocomplete="">"#)]
    #[case(r#"<input autocomplete=" ">"#)]
    #[case(r#"<input autocomplete="badname" hidden>"#)]
    #[case(r#"<input autocomplete="badname" disabled>"#)]
    #[case(r#"<input autocomplete="badname" aria-disabled="true">"#)]
    #[case(r#"<input type="text" role="none" disabled autocomplete="badname">"#)]
    #[case(r#"<input type="text" autocomplete="OFF">"#)]
    #[case(r#"<input type="submit" autocomplete="email">"#)]
    #[case(r#"<input type="hidden" autocomplete="transaction-amount">"#)]
    #[case(r#"<div autocomplete="badname"></div>"#)]
    fn valid_or_inapplicable_values_pass(#[case] html: &str) {
        assert_eq!(count(html), 0, "{html}");
    }
}
