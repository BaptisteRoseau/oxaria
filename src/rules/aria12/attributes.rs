//! Which states and properties an element may carry: defined names
//! (ARIA-ATTR001), supported (ARIA-ATTR003) and prohibited (ARIA-ATTR004)
//! ones, table-specific restrictions (ARIA-ATTR005) and
//! `aria-roledescription` (ARIA-ATTR008).

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    ARIA_1_3_ATTRIBUTES, Parsed, Role, Scope, aria_attributes, attribute, attributes,
    effective_role, parse_value, roles,
};
use crate::rules::{CheckOptions, Finding};

use super::support::{closest, has_value, is_unroled_custom_element, quoted, role_label};
use super::tree::exposed_role;

const TREEGRID_ROW_ATTRIBUTES: &[&str] = &[
    "aria-expanded",
    "aria-level",
    "aria-posinset",
    "aria-setsize",
];
const EDITABLE_GRID_HEADER_ATTRIBUTES: &[&str] = &["aria-readonly", "aria-required"];
const NATIVE_SPAN_ATTRIBUTES: &[(&str, &str)] =
    &[("aria-colspan", "colspan"), ("aria-rowspan", "rowspan")];

/// ARIA-ATTR001: an `aria-*` name WAI-ARIA doesn't define does nothing.
/// The 1.3 draft's names are left alone: browsers already support some.
pub fn check_unknown_attribute(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            aria_attributes(el)
                .filter(|(name, _)| {
                    attribute(name).is_none() && !ARIA_1_3_ATTRIBUTES.contains(name)
                })
                .map(move |(name, _)| {
                    Finding::error(
                        "ARIA-ATTR001",
                        format!("{name} is not a WAI-ARIA 1.2 attribute"),
                    )
                    .at(el)
                    .help(unknown_attribute_help(name))
                })
        })
        .collect()
}

fn unknown_attribute_help(name: &str) -> String {
    let known = attributes().map(|attribute| attribute.name);
    match (name, closest(name, known)) {
        ("aria-role", _) => "use the `role` attribute: aria-role does nothing".to_string(),
        (_, Some(suggestion)) => format!("did you mean `{suggestion}`?"),
        _ => format!("remove {name}, or replace it with a WAI-ARIA 1.2 state or property"),
    }
}

/// ARIA-ATTR003: a role-specific state or property on a role that doesn't
/// support it is ignored.
pub fn check_unsupported_attribute(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_unroled_custom_element(*el))
        .flat_map(unsupported_attribute_findings)
        .collect()
}

fn unsupported_attribute_findings(el: ElementRef) -> Vec<Finding> {
    // A separator supports the range properties only when focusable, which
    // the role table can't express.
    let Some(role) = effective_role(el).filter(|role| role.name != "separator") else {
        return Vec::new();
    };
    aria_attributes(el)
        .filter(|(name, value)| is_role_specific(name, value) && !role.supports(name))
        .map(|(name, value)| {
            Finding::error(
                "ARIA-ATTR003",
                format!(
                    "{} is not supported on {}",
                    quoted(name, value),
                    role_label(el, role)
                ),
            )
            .at(el)
            .help(unsupported_attribute_help(name))
        })
        .collect()
}

fn is_role_specific(name: &str, value: &str) -> bool {
    attribute(name).is_some_and(|attribute| {
        attribute.scope == Scope::RoleSpecific && parse_value(attribute, value) != Parsed::Empty
    })
}

fn unsupported_attribute_help(name: &str) -> String {
    let supporting: Vec<&str> = roles()
        .filter(|role| !role.is_abstract && role.specific_attributes.contains(&name))
        .map(|role| role.name)
        .collect();
    format!(
        "remove {name}, or use it on an element with a role that supports it: {}",
        supporting.join(", ")
    )
}

/// ARIA-ATTR004: a prohibited state or property, such as a name on a
/// `generic` element, is ignored.
pub fn check_prohibited_attribute(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_unroled_custom_element(*el))
        .flat_map(prohibited_attribute_findings)
        .collect()
}

fn prohibited_attribute_findings(el: ElementRef) -> Vec<Finding> {
    let Some(role) = effective_role(el) else {
        return Vec::new();
    };
    aria_attributes(el)
        .filter(|(name, value)| role.prohibits(name) && !value.trim().is_empty())
        .map(|(name, value)| {
            Finding::error(
                "ARIA-ATTR004",
                format!(
                    "{} is prohibited on {}",
                    quoted(name, value),
                    role_label(el, role)
                ),
            )
            .at(el)
            .help(prohibited_attribute_help(name, role))
        })
        .collect()
}

fn prohibited_attribute_help(name: &str, role: &Role) -> String {
    match (name, role.name) {
        ("aria-roledescription", _) => {
            "remove aria-roledescription, or give the element a role it can describe, such as \
             role=\"group\" or role=\"region\""
                .to_string()
        }
        (_, "generic") => format!(
            "give the element a role that can be named, such as role=\"group\" or \
             role=\"region\", or remove {name} and put the text in the content"
        ),
        _ => format!(
            "remove {name}: a {} can't be named, so put the text in its content",
            role.name
        ),
    }
}

/// ARIA-ATTR005: hierarchy attributes belong to `treegrid` rows, editing
/// attributes to `grid` headers, and native tables span with
/// `colspan`/`rowspan`.
pub fn check_static_table_attribute(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all().flat_map(static_table_findings).collect()
}

fn static_table_findings(el: ElementRef) -> Vec<Finding> {
    let restricted = match exposed_role(el).map(|role| role.name) {
        Some("row") => restricted_row_attributes(el),
        Some("columnheader" | "rowheader") => restricted_header_attributes(el),
        _ => Vec::new(),
    };
    restricted
        .into_iter()
        .chain(native_span_findings(el))
        .collect()
}

fn restricted_row_attributes(row: ElementRef) -> Vec<Finding> {
    let Some(table) = table_role(row).filter(|role| matches!(*role, "table" | "grid")) else {
        return Vec::new();
    };
    TREEGRID_ROW_ATTRIBUTES
        .iter()
        .filter(|name| has_value(row, name))
        .map(|name| {
            Finding::error(
                "ARIA-ATTR005",
                format!(
                    "{} is set on a row of a {table}",
                    quoted(name, row.attr(name).unwrap_or_default())
                ),
            )
            .at(row)
            .help(format!(
                "remove {name}: rows only expand and nest in a treegrid (use role=\"treegrid\" \
                 if the rows really form a hierarchy)"
            ))
        })
        .collect()
}

fn restricted_header_attributes(header: ElementRef) -> Vec<Finding> {
    if table_role(header) != Some("table") {
        return Vec::new();
    }
    EDITABLE_GRID_HEADER_ATTRIBUTES
        .iter()
        .filter(|name| has_value(header, name))
        .map(|name| {
            Finding::error(
                "ARIA-ATTR005",
                format!(
                    "{} is set on a header of a static table",
                    quoted(name, header.attr(name).unwrap_or_default())
                ),
            )
            .at(header)
            .help(format!(
                "remove {name}: a table's content isn't editable (use role=\"grid\" if it is)"
            ))
        })
        .collect()
}

fn native_span_findings(el: ElementRef) -> Vec<Finding> {
    let is_native_cell =
        matches!(el.tag(), "td" | "th") && el.ancestors().any(|ancestor| ancestor.tag() == "table");
    if !is_native_cell {
        return Vec::new();
    }
    NATIVE_SPAN_ATTRIBUTES
        .iter()
        .filter(|(name, _)| has_value(el, name))
        .map(|(name, native)| {
            let value = el.attr(name).unwrap_or_default().trim();
            Finding::error(
                "ARIA-ATTR005",
                format!(
                    "<{}> in a native table uses {}",
                    el.tag(),
                    quoted(name, value)
                ),
            )
            .at(el)
            .help(format!("use {native}=\"{value}\" instead of {name}"))
        })
        .collect()
}

/// The role of the nearest ancestor exposed as `table`, `grid` or
/// `treegrid`.
fn table_role(el: ElementRef) -> Option<&'static str> {
    el.ancestors()
        .filter_map(exposed_role)
        .map(|role| role.name)
        .find(|name| matches!(*name, "table" | "grid" | "treegrid"))
}

/// ARIA-ATTR008: `aria-roledescription` only describes an element with a
/// role, and an empty one erases the role's description.
pub fn check_role_description(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_unroled_custom_element(*el))
        .filter_map(|el| {
            let value = el.attr("aria-roledescription")?;
            role_description_problem(el, value)
                .map(|(message, help)| Finding::error("ARIA-ATTR008", message).at(el).help(help))
        })
        .collect()
}

fn role_description_problem(el: ElementRef, value: &str) -> Option<(String, &'static str)> {
    let role = effective_role(el).filter(|role| role.name != "generic");
    match (value.trim().is_empty(), role) {
        (true, _) => Some((
            format!("aria-roledescription=\"{value}\" is empty"),
            "describe the role in words (e.g. aria-roledescription=\"slide\"), or remove the \
             attribute",
        )),
        (false, None) => Some((
            format!(
                "{} is set on <{}>, which has no role to describe",
                quoted("aria-roledescription", value),
                el.tag()
            ),
            "give the element a role it describes, such as role=\"region\" or role=\"group\" \
             (with a name), or remove aria-roledescription",
        )),
        (false, Some(_)) => None,
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
    #[case(
        r#"<input type="text" aria-labeledby="email-label">"#,
        "did you mean `aria-labelledby`?"
    )]
    #[case(
        r#"<div aria-role="button" tabindex="0">Save</div>"#,
        "use the `role` attribute"
    )]
    #[case(r#"<div aria-foo="1">x</div>"#, "remove aria-foo")]
    fn unknown_attributes_are_flagged(#[case] html: &str, #[case] help: &str) {
        let findings = run(check_unknown_attribute, html);
        assert_eq!(findings.len(), 1);
        assert!(
            findings[0].help.as_deref().unwrap().starts_with(help),
            "{findings:?}"
        );
    }

    #[rstest]
    #[case(r#"<input type="text" aria-labelledby="email-label">"#)]
    #[case(r#"<div role="button" tabindex="0">Save</div>"#)]
    #[case(r#"<button aria-description="Saves the draft">Save</button>"#)]
    fn known_attributes_are_not_flagged(#[case] html: &str) {
        assert!(run(check_unknown_attribute, html).is_empty());
    }

    #[rstest]
    #[case(r#"<div role="button" tabindex="0" aria-checked="true">Bold</div>"#)]
    #[case(r#"<input type="text" aria-selected="true">"#)]
    #[case(r#"<div aria-expanded="false">Menu</div>"#)]
    #[case(r#"<ul><li aria-selected="true">x</li></ul>"#)]
    fn unsupported_attributes_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_unsupported_attribute, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<div role="button" tabindex="0" aria-pressed="true">Bold</div>"#)]
    #[case(r#"<input type="text" aria-required="true">"#)]
    #[case(r#"<input type="password" aria-required="true">"#)]
    #[case(r#"<input type="checkbox" aria-checked="true">"#)]
    #[case(r#"<div role="separator" aria-valuenow="30"></div>"#)]
    #[case(r#"<div aria-expanded="">Menu</div>"#)]
    #[case(r#"<my-tree aria-expanded="true">x</my-tree>"#)]
    #[case(r#"<div aria-label="x" aria-live="polite">x</div>"#)]
    fn supported_attributes_are_not_flagged(#[case] html: &str) {
        assert!(run(check_unsupported_attribute, html).is_empty(), "{html}");
    }

    #[test]
    fn unsupported_attribute_help_lists_supporting_roles() {
        let findings = run(
            check_unsupported_attribute,
            r#"<a href="/" aria-pressed="true">x</a>"#,
        );
        assert_eq!(
            findings[0].help.as_deref(),
            Some(
                "remove aria-pressed, or use it on an element with a role that supports it: button"
            )
        );
        assert_eq!(
            findings[0].message,
            "aria-pressed=\"true\" is not supported on role \"link\" (implicit on <a>)"
        );
    }

    #[rstest]
    #[case(r#"<div class="card" aria-label="Product: Trail running shoes">...</div>"#)]
    #[case(r#"<p aria-label="Warning">Your session expires in 5 minutes.</p>"#)]
    #[case(r#"<span role="none" aria-labelledby="x">x</span><i id="x">x</i>"#)]
    fn prohibited_attributes_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_prohibited_attribute, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<div class="card" role="group" aria-label="Product: Trail running shoes">...</div>"#)]
    #[case("<p><strong>Warning:</strong> your session expires in 5 minutes.</p>")]
    #[case(r#"<nav aria-label="Main">x</nav>"#)]
    #[case(r#"<div aria-label="">x</div>"#)]
    #[case(r#"<fancy-card aria-label="Card">x</fancy-card>"#)]
    fn allowed_names_are_not_flagged(#[case] html: &str) {
        assert!(run(check_prohibited_attribute, html).is_empty(), "{html}");
    }

    #[rstest]
    #[case(
        r#"<div role="table"><div role="row" aria-level="1" aria-expanded="true">x</div></div>"#,
        2
    )]
    #[case(r#"<table><tr><td aria-colspan="2">Total</td></tr></table>"#, 1)]
    #[case(r#"<table><tr><th aria-readonly="true">Name</th></tr></table>"#, 1)]
    #[case(r#"<div role="grid"><div role="rowgroup"><div role="row" aria-setsize="3">x</div></div></div>"#, 1)]
    fn static_table_attributes_are_flagged(#[case] html: &str, #[case] count: usize) {
        assert_eq!(
            run(check_static_table_attribute, html).len(),
            count,
            "{html}"
        );
    }

    #[rstest]
    #[case(
        r#"<div role="treegrid"><div role="row" aria-level="1" aria-expanded="true">x</div></div>"#
    )]
    #[case(r#"<table><tr><td colspan="2">Total</td></tr></table>"#)]
    #[case(r#"<div role="grid"><div role="row"><div role="columnheader" aria-readonly="true">N</div></div></div>"#)]
    #[case(r#"<div role="table"><div role="row"><div role="cell" aria-colspan="2">x</div></div></div>"#)]
    fn allowed_table_attributes_are_not_flagged(#[case] html: &str) {
        assert!(run(check_static_table_attribute, html).is_empty(), "{html}");
    }

    #[test]
    fn native_span_help_gives_the_native_attribute() {
        let findings = run(
            check_static_table_attribute,
            r#"<table><tr><td aria-colspan="2">Total</td></tr></table>"#,
        );
        assert_eq!(
            findings[0].help.as_deref(),
            Some("use colspan=\"2\" instead of aria-colspan")
        );
    }

    #[rstest]
    #[case(r#"<div aria-roledescription="slide" id="slide42">...</div>"#)]
    #[case(r#"<div role="region" aria-roledescription=" " aria-labelledby="h">...</div>"#)]
    #[case(r#"<svg aria-roledescription="chart"></svg>"#)]
    fn misused_role_descriptions_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_role_description, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<div role="region" aria-roledescription="slide" aria-labelledby="h"><h1 id="h">Q</h1></div>"#)]
    #[case(r#"<button aria-roledescription="toggle">x</button>"#)]
    fn role_descriptions_on_roles_are_not_flagged(#[case] html: &str) {
        assert!(run(check_role_description, html).is_empty(), "{html}");
    }
}
