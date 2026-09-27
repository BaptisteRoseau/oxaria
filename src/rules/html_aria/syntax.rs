//! ARIA in HTML's case requirement on `role` and `aria-*` token values.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{Parsed, ValueType, aria_attributes, attribute, parse_value, role};
use crate::rules::{CheckOptions, Finding};

use super::markup::start_tag;

/// HTMLARIA016: role tokens and `aria-*` token values not written in
/// ASCII lowercase. Only tokens browsers would recognize once lowercased
/// are reported; unknown ones are WAI-ARIA's rules' business.
pub fn check_lowercase_tokens(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| uppercase_role(el).into_iter().chain(uppercase_values(el)))
        .collect()
}

fn uppercase_role(el: ElementRef) -> Option<Finding> {
    let value = el.attr("role")?;
    let has_uppercase_role = value
        .split_ascii_whitespace()
        .any(|token| has_uppercase(token) && is_known_role(&token.to_ascii_lowercase()));
    has_uppercase_role.then(|| {
        Finding::error(
            "HTMLARIA016",
            format!(
                "{} writes a role token with uppercase letters",
                start_tag(el, &["role"])
            ),
        )
        .at(el)
        .help(lowercase_help("role", value))
    })
}

fn is_known_role(token: &str) -> bool {
    role(token).is_some() || token.starts_with("doc-") || token.starts_with("graphics-")
}

fn uppercase_values(el: ElementRef<'_>) -> impl Iterator<Item = Finding> + '_ {
    aria_attributes(el)
        .filter(|(name, value)| has_uppercase(value) && is_valid_token_value(name, value))
        .map(move |(name, value)| {
            Finding::error(
                "HTMLARIA016",
                format!(
                    "{} writes a token value with uppercase letters",
                    start_tag(el, &[name])
                ),
            )
            .at(el)
            .help(lowercase_help(name, value))
        })
}

fn is_valid_token_value(name: &str, value: &str) -> bool {
    attribute(name).is_some_and(|attribute| {
        matches!(
            attribute.value_type,
            ValueType::Token | ValueType::TokenList
        ) && matches!(parse_value(attribute, value), Parsed::Valid(_))
    })
}

fn has_uppercase(value: &str) -> bool {
    value.chars().any(|c| c.is_ascii_uppercase())
}

fn lowercase_help(name: &str, value: &str) -> String {
    format!(
        "write {name}=\"{}\" in lowercase",
        value.trim().to_ascii_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(html: &str) -> Vec<Finding> {
        check_lowercase_tokens(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(r#"<div role="MAIN">x</div>"#, r#"write role="main""#)]
    #[case(
        r#"<a href="home/" aria-current="Page">home</a>"#,
        r#"write aria-current="page""#
    )]
    #[case(
        r#"<div role="bogus Doc-Toc">x</div>"#,
        r#"write role="bogus doc-toc""#
    )]
    #[case(
        r#"<div aria-relevant="additions TEXT">x</div>"#,
        r#"write aria-relevant="additions text""#
    )]
    fn uppercase_tokens_are_flagged(#[case] html: &str, #[case] help: &str) {
        let findings = run(html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA016");
        assert!(findings[0].help.as_deref().unwrap().starts_with(help));
    }

    #[rstest]
    #[case(r#"<div role="main">x</div>"#)]
    #[case(r#"<a href="home/" aria-current="page">home</a>"#)]
    #[case(r#"<div role="Bogus">x</div>"#)]
    #[case(r#"<div aria-live="Loud">x</div>"#)]
    #[case(r#"<button aria-label="Close Menu">x</button>"#)]
    #[case(r#"<div aria-hidden="True">x</div>"#)]
    fn lowercase_or_non_token_values_are_not_flagged(#[case] html: &str) {
        assert!(run(html).is_empty());
    }
}
