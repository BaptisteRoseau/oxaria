//! WCAG 3.3.8 (Accessible Authentication (Minimum)) check for paste blocking.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

const SECRET_AUTOCOMPLETE: &[&str] = &["current-password", "new-password", "one-time-code"];

/// AUT001: blocking paste into a password or code field stops password managers and forces
/// users to transcribe it, which the Understanding document for 3.3.8 calls out as a failure.
pub fn check_paste_blocked(page: &RenderedPage) -> Vec<Finding> {
    page.by_tag("input")
        .filter(|input| is_secret_field(*input))
        .filter(|input| input.attr("onpaste").is_some_and(cancels_event))
        .map(|input| {
            Finding::error(
                "AUT001",
                "password or code field blocks pasting".to_string(),
            )
            .at(input)
            .help("remove the onpaste handler so users can paste from a password manager")
        })
        .collect()
}

fn is_secret_field(input: ElementRef) -> bool {
    input.attr("type") == Some("password")
        || input.attr("autocomplete").is_some_and(|value| {
            value
                .split_ascii_whitespace()
                .any(|token| SECRET_AUTOCOMPLETE.contains(&token))
        })
}

fn cancels_event(code: &str) -> bool {
    let code = code.replace(' ', "");
    code.contains("returnfalse") || code.contains("preventDefault(")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::rule_ids;

    #[rstest]
    #[case(
        r#"<label for="pwd">Password</label><input id="pwd" type="password" onpaste="return false">"#,
        &["AUT001"]
    )]
    #[case(
        r#"<input autocomplete="one-time-code" onpaste="event.preventDefault()">"#,
        &["AUT001"]
    )]
    // A password field allowing paste.
    #[case(
        r#"<input type="password" autocomplete="current-password" onpaste="track()">"#,
        &[]
    )]
    // Other fields blocking paste.
    #[case(r#"<input type="email" onpaste="return false">"#, &[])]
    fn paste_blocked(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_paste_blocked, html), expected, "{html}");
    }
}
