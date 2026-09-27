//! ACT rules for `aria-*` attribute names and values and `role` values.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    ARIA_1_3_ATTRIBUTES, Attribute, Parsed, ValueType, aria_attributes, attribute, attributes,
    concrete_role, explicit_roles, is_hidden, parse_value, roles,
};
use crate::rules::{CheckOptions, Finding};

/// The Graphics-ARIA and DPub-ARIA 1.1 roles: ACT accepts every WAI-ARIA
/// specification, and the shared role table only holds WAI-ARIA 1.2.
const MODULE_ROLES: &[&str] = &[
    "graphics-document",
    "graphics-object",
    "graphics-symbol",
    "doc-abstract",
    "doc-acknowledgments",
    "doc-afterword",
    "doc-appendix",
    "doc-backlink",
    "doc-biblioentry",
    "doc-bibliography",
    "doc-biblioref",
    "doc-chapter",
    "doc-colophon",
    "doc-conclusion",
    "doc-cover",
    "doc-credit",
    "doc-credits",
    "doc-dedication",
    "doc-endnote",
    "doc-endnotes",
    "doc-epigraph",
    "doc-epilogue",
    "doc-errata",
    "doc-example",
    "doc-footnote",
    "doc-foreword",
    "doc-glossary",
    "doc-glossref",
    "doc-index",
    "doc-introduction",
    "doc-noteref",
    "doc-notice",
    "doc-pagebreak",
    "doc-pagefooter",
    "doc-pageheader",
    "doc-pagelist",
    "doc-part",
    "doc-preface",
    "doc-prologue",
    "doc-pullquote",
    "doc-qna",
    "doc-subtitle",
    "doc-tip",
    "doc-toc",
];

/// Typos further away than this get no "did you mean" suggestion.
const MAX_SUGGESTION_DISTANCE: usize = 2;

/// 5f99a7: every `aria-*` attribute must be defined in a WAI-ARIA
/// specification.
pub fn check_aria_attribute_defined(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            aria_attributes(el)
                .filter(|(name, _)| !is_defined_attribute(name))
                .map(move |(name, value)| undefined_attribute_finding(el, name, value))
        })
        .collect()
}

/// WAI-ARIA 1.3 attributes are accepted: browsers already support some.
fn is_defined_attribute(name: &str) -> bool {
    attribute(name).is_some() || ARIA_1_3_ATTRIBUTES.contains(&name)
}

fn undefined_attribute_finding(el: ElementRef, name: &str, value: &str) -> Finding {
    let candidates = attributes()
        .map(|attribute| attribute.name)
        .chain(ARIA_1_3_ATTRIBUTES.iter().copied());
    let help = match closest(name, candidates) {
        Some(known) => format!("did you mean {known}=\"{value}\"? otherwise remove it"),
        None => format!("{name} is ignored by browsers: remove it, or use a WAI-ARIA attribute"),
    };
    Finding::error(
        "5f99a7",
        format!(
            "<{} {name}=\"{value}\"> uses an attribute WAI-ARIA doesn't define",
            el.tag()
        ),
    )
    .at(el)
    .help(help)
}

/// 6a7281: a non-empty ARIA state or property must have a value valid for
/// its value type.
pub fn check_aria_value_valid(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            aria_attributes(el)
                .filter_map(|(name, value)| Some((attribute(name)?, value)))
                .filter(|(attribute, value)| parse_value(attribute, value) == Parsed::Invalid)
                .map(move |(attribute, value)| invalid_value_finding(el, attribute, value))
        })
        .collect()
}

fn invalid_value_finding(el: ElementRef, attribute: &Attribute, value: &str) -> Finding {
    Finding::error(
        "6a7281",
        format!(
            "<{} {}=\"{value}\"> has a value that isn't a valid {}",
            el.tag(),
            attribute.name,
            value_type_name(attribute.value_type)
        ),
    )
    .at(el)
    .help(format!(
        "set {} to {}",
        attribute.name,
        expected_value(attribute)
    ))
}

fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::TrueFalse => "true/false",
        ValueType::TrueFalseUndefined => "true/false/undefined",
        ValueType::Tristate => "tristate",
        ValueType::IdRef => "ID reference",
        ValueType::IdRefList => "ID reference list",
        ValueType::Integer => "integer",
        ValueType::Number => "number",
        ValueType::String => "string",
        ValueType::Token => "token",
        ValueType::TokenList => "token list",
    }
}

fn expected_value(attribute: &Attribute) -> String {
    let values = attribute
        .values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>();
    match attribute.value_type {
        ValueType::Integer => "a whole number, e.g. \"2\"".to_string(),
        ValueType::Number => "a number, e.g. \"2.5\"".to_string(),
        ValueType::IdRef => "a single id, without spaces".to_string(),
        ValueType::TokenList => format!("space-separated tokens among {}", values.join(", ")),
        _ => format!("one of {}", values.join(", ")),
    }
}

/// 674b10: a non-empty `role` attribute on an element that isn't hidden
/// must contain at least one valid, non-abstract role.
pub fn check_role_valid(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !explicit_roles(*el).is_empty() && !is_hidden(*el))
        .filter(|el| !explicit_roles(*el).iter().any(|token| is_valid_role(token)))
        .map(|el| {
            let value = el.attr("role").unwrap_or_default();
            Finding::error(
                "674b10",
                format!(
                    "<{} role=\"{value}\"> contains no valid WAI-ARIA role",
                    el.tag()
                ),
            )
            .at(el)
            .help(role_help(el))
        })
        .collect()
}

fn is_valid_role(token: &str) -> bool {
    concrete_role(token).is_some() || MODULE_ROLES.contains(&token)
}

fn role_help(el: ElementRef) -> String {
    let candidates: Vec<&str> = roles()
        .filter(|role| !role.is_abstract)
        .map(|role| role.name)
        .chain(MODULE_ROLES.iter().copied())
        .collect();
    let suggestion = explicit_roles(el)
        .iter()
        .find_map(|token| closest(token, candidates.iter().copied()));
    match suggestion {
        Some(role) => format!("did you mean role=\"{role}\"?"),
        None => "use a non-abstract WAI-ARIA role (e.g. role=\"button\"), \
                 or remove the attribute"
            .to_string(),
    }
}

fn closest<'a>(name: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    candidates
        .map(|candidate| (edit_distance(name, candidate), candidate))
        .filter(|(distance, _)| *distance <= MAX_SUGGESTION_DISTANCE)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, a_char) in a.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, b_char) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(a_char != *b_char);
            current.push(substitution.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;
    use crate::rules::RuleCheck;

    fn findings(check: RuleCheck, html: &str) -> Vec<Finding> {
        check(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(r#"<div role="checkbox" aria-not-checked="true">All met</div>"#, 1)]
    #[case(
        r#"<div role="searchbox" aria-labelled="l" aria-placeholder="MM-DD">x</div>"#,
        1
    )]
    #[case(r#"<article aria-atomic="true">This is cool</article>"#, 0)]
    #[case(
        r#"<div role="dialog" aria-modal="true" aria-label="Modal">x</div>"#,
        0
    )]
    #[case(
        r#"<input aria-valuemax="100" aria-valuemin="0" aria-valuenow="25" type="number">"#,
        0
    )]
    #[case(r#"<div aria-description="More">x</div>"#, 0)]
    #[case("<canvas></canvas>", 0)]
    fn undefined_attributes_5f99a7(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(
            findings(check_aria_attribute_defined, html).len(),
            expected,
            "{html}"
        );
    }

    #[test]
    fn typos_get_a_suggestion() {
        let found = findings(
            check_aria_attribute_defined,
            r#"<div aria-labelled="l">x</div>"#,
        );
        assert_eq!(
            found[0].help.as_deref(),
            Some(r#"did you mean aria-labelledby="l"? otherwise remove it"#)
        );
    }

    #[rstest]
    #[case(
        r#"<div role="textbox" aria-required="undefined" aria-label="A">x</div>"#,
        1
    )]
    #[case(r#"<div role="button" aria-expanded="collapsed">A button</div>"#, 1)]
    #[case(r#"<div role="button" aria-pressed="horizontal">B</div>"#, 1)]
    #[case(r#"<div role="gridcell" aria-rowindex="2.5">Fred</div>"#, 1)]
    #[case(r#"<div role="spinbutton" aria-valuemin="one" aria-valuemax="three" aria-valuenow="two" aria-label="C"></div>"#, 3)]
    #[case(r#"<div role="main" aria-live="page"></div>"#, 1)]
    #[case(r#"<div role="alert" aria-relevant="text always"></div>"#, 1)]
    #[case(r#"<div role="textbox" aria-label="Family name"></div>"#, 0)]
    #[case(
        r#"<div role="textbox" aria-required="true" aria-label="Family name"></div>"#,
        0
    )]
    #[case(r#"<div role="button" aria-expanded="undefined">A button</div>"#, 0)]
    #[case(r#"<div role="button" aria-pressed="mixed">Partially</div>"#, 0)]
    #[case(
        r#"<div role="textbox" aria-errormessage="my-error" aria-label="A"></div>"#,
        0
    )]
    #[case(r#"<div role="list" aria-owns="item1 item2"></div>"#, 0)]
    #[case(r#"<div role="gridcell" aria-rowindex="2">Fred</div>"#, 0)]
    #[case(r#"<div role="spinbutton" aria-valuemin="1.0" aria-valuemax="2.0" aria-valuenow="1.5"></div>"#, 0)]
    #[case(r#"<a href="/" aria-current="page">Home</a>"#, 0)]
    #[case(r#"<div role="alert" aria-relevant="text removals"></div>"#, 0)]
    #[case(r#"<div role="alert" aria-live>Be awesome</div>"#, 0)]
    #[case(r#"<div aria-bogus="nope">x</div>"#, 0)]
    fn values_6a7281(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(
            findings(check_aria_value_valid, html).len(),
            expected,
            "{html}"
        );
    }

    #[test]
    fn value_help_lists_the_allowed_values() {
        let found = findings(
            check_aria_value_valid,
            r#"<div role="button" aria-expanded="collapsed">A</div>"#,
        );
        assert_eq!(
            found[0].message,
            r#"<div aria-expanded="collapsed"> has a value that isn't a valid true/false/undefined"#
        );
        assert_eq!(
            found[0].help.as_deref(),
            Some(r#"set aria-expanded to one of "false", "true", "undefined""#)
        );
    }

    #[rstest]
    #[case(r#"I love <span onclick="go()" role="lnik">ACT rules</span>."#, 1)]
    #[case(r#"<span role="bibliographic-reference lnik">ACT rules</span>"#, 1)]
    #[case(r#"<div role="widget">x</div>"#, 1)]
    #[case(r#"<input type="text" role="searchbox">"#, 0)]
    #[case(r#"<span role="doc-biblioref link">ACT rules</span>"#, 0)]
    #[case(r#"<input type="text" role="searchfield searchbox">"#, 0)]
    #[case(r#"<svg role="graphics-document"></svg>"#, 0)]
    #[case(r#"<div role="image">x</div>"#, 0)]
    #[case(r#"<img src="w3c.png" alt="W3C logo">"#, 0)]
    #[case("<div role>Some Content</div>", 0)]
    #[case(r#"<div role="">Some Content</div>"#, 0)]
    #[case(r#"<input type="text" role=" " aria-label="field">"#, 0)]
    #[case(r#"<div aria-hidden="true" role="bannner">Some Content</div>"#, 0)]
    fn roles_674b10(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(findings(check_role_valid, html).len(), expected, "{html}");
    }

    #[test]
    fn role_typos_get_a_suggestion() {
        let found = findings(check_role_valid, r#"<span role="lnik">ACT</span>"#);
        assert_eq!(
            found[0].help.as_deref(),
            Some(r#"did you mean role="link"?"#)
        );
    }

    #[rstest]
    #[case("lnik", "link", 2)]
    #[case("aria-labelled", "aria-labelledby", 2)]
    #[case("", "abc", 3)]
    #[case("same", "same", 0)]
    fn edit_distances(#[case] a: &str, #[case] b: &str, #[case] expected: usize) {
        assert_eq!(edit_distance(a, b), expected);
    }
}
