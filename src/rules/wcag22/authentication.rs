//! WCAG 3.3.8 (Accessible Authentication (Minimum)) check for paste blocking.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::{CheckOptions, Finding};

const SECRET_AUTOCOMPLETE: &[&str] = &["current-password", "new-password", "one-time-code"];

/// AUT001: blocking paste into a password or code field stops password managers and forces
/// users to transcribe it, which the Understanding document for 3.3.8 calls out as a failure.
pub fn check_paste_blocked(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
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
    use super::*;
    use crate::page::testutil::page_from_html;

    fn findings(html: &str) -> Vec<Finding> {
        check_paste_blocked(&page_from_html(html), &CheckOptions::default())
    }

    #[test]
    fn password_field_blocking_paste_is_flagged() {
        let findings = findings(
            r#"<label for="pwd">Password</label><input id="pwd" type="password" onpaste="return false">"#,
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "AUT001");
    }

    #[test]
    fn code_field_blocking_paste_is_flagged() {
        let html = r#"<input autocomplete="one-time-code" onpaste="event.preventDefault()">"#;
        assert_eq!(findings(html).len(), 1);
    }

    #[test]
    fn password_field_allowing_paste_is_not_flagged() {
        let html = r#"<input type="password" autocomplete="current-password" onpaste="track()">"#;
        assert!(findings(html).is_empty());
    }

    #[test]
    fn other_fields_blocking_paste_are_not_flagged() {
        assert!(findings(r#"<input type="email" onpaste="return false">"#).is_empty());
    }
}
