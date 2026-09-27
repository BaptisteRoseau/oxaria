//! Features WAI-ARIA 1.2 deprecates: the `directory` role (ARIA-DEPR001),
//! drag-and-drop attributes (ARIA-DEPR002), and the global use of four
//! attributes (ARIA-DEPR003).

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{effective_role, explicit_roles};
use crate::rules::{CheckOptions, Finding};

use super::support::{has_value, is_unroled_custom_element, quoted, role_label};

const DRAG_AND_DROP_ATTRIBUTES: &[&str] = &["aria-grabbed", "aria-dropeffect"];

/// Deprecated as globals in 1.2; still supported by some roles.
const FORMERLY_GLOBAL_ATTRIBUTES: &[&str] = &[
    "aria-disabled",
    "aria-errormessage",
    "aria-haspopup",
    "aria-invalid",
];

/// ARIA-DEPR001: `directory` is deprecated in favor of `list`.
pub fn check_directory_role(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| explicit_roles(*el).iter().any(|token| token == "directory"))
        .map(|el| {
            Finding::error(
                "ARIA-DEPR001",
                format!("<{}> uses the deprecated role=\"directory\"", el.tag()),
            )
            .at(el)
            .help("use role=\"list\", or a native <ul> or <ol> list, instead")
        })
        .collect()
}

/// ARIA-DEPR002: `aria-grabbed`/`aria-dropeffect` are deprecated and
/// poorly supported.
pub fn check_drag_and_drop_attributes(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            DRAG_AND_DROP_ATTRIBUTES
                .iter()
                .filter(move |name| has_value(el, name))
                .map(move |name| {
                    Finding::error(
                        "ARIA-DEPR002",
                        format!(
                            "{} uses a deprecated drag-and-drop attribute",
                            quoted(name, el.attr(name).unwrap_or_default())
                        ),
                    )
                    .at(el)
                    .help(format!(
                        "remove {name}, and give keyboard and screen reader users another way \
                         to move the item, such as a \"Move to\" button"
                    ))
                })
        })
        .collect()
}

/// ARIA-DEPR003: `aria-disabled`, `aria-errormessage`, `aria-haspopup` and
/// `aria-invalid` are deprecated on roles that don't list them.
pub fn check_deprecated_global_attributes(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_unroled_custom_element(*el))
        .flat_map(deprecated_attribute_findings)
        .collect()
}

fn deprecated_attribute_findings(el: ElementRef) -> Vec<Finding> {
    let Some(role) = effective_role(el) else {
        return Vec::new();
    };
    FORMERLY_GLOBAL_ATTRIBUTES
        .iter()
        .filter(|name| has_value(el, name) && role.deprecates(name))
        .map(|name| {
            Finding::error(
                "ARIA-DEPR003",
                format!(
                    "{} is deprecated on {}",
                    quoted(name, el.attr(name).unwrap_or_default()),
                    role_label(el, role)
                ),
            )
            .at(el)
            .help(deprecated_attribute_help(name))
        })
        .collect()
}

fn deprecated_attribute_help(name: &str) -> String {
    let fix = match name {
        "aria-disabled" => {
            "use it on the control itself, or disable native form controls with `disabled` \
             (a <fieldset disabled> disables all of them)"
        }
        "aria-haspopup" => "put it on the button or link that opens the popup",
        _ => "put it on the form field whose value is invalid",
    };
    format!("remove {name} from this element: {fix}")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(check: fn(&RenderedPage, &CheckOptions) -> Vec<Finding>, html: &str) -> Vec<Finding> {
        check(&page_from_html(html), &CheckOptions::default())
    }

    #[test]
    fn directory_role_is_flagged() {
        let html = r##"<ul role="directory"><li><a href="#intro">Introduction</a></li></ul>"##;
        assert_eq!(run(check_directory_role, html).len(), 1);
        let html = r##"<ul><li><a href="#intro">Introduction</a></li></ul>"##;
        assert!(run(check_directory_role, html).is_empty());
    }

    #[test]
    fn drag_and_drop_attributes_are_flagged() {
        let html = r#"<li draggable="true" aria-grabbed="false">Task: write report</li>
                      <ul aria-dropeffect="move" aria-label="Done"></ul>"#;
        assert_eq!(run(check_drag_and_drop_attributes, html).len(), 2);
        let html = r#"<li draggable="true">Task <button type="button">Move to Done</button></li>"#;
        assert!(run(check_drag_and_drop_attributes, html).is_empty());
    }

    #[rstest]
    #[case(r#"<div role="img" aria-label="Profile photo" aria-haspopup="menu"></div>"#)]
    #[case(r#"<section aria-disabled="true">...</section>"#)]
    #[case(r#"<a aria-disabled="true">Export</a>"#)]
    #[case(r#"<h2 aria-invalid="true">Title</h2>"#)]
    fn deprecated_globals_are_flagged(#[case] html: &str) {
        assert_eq!(
            run(check_deprecated_global_attributes, html).len(),
            1,
            "{html}"
        );
    }

    #[rstest]
    #[case(
        r#"<button aria-haspopup="menu" aria-label="Options"><img src="me.jpg" alt=""></button>"#
    )]
    #[case("<fieldset disabled>...</fieldset>")]
    #[case(r#"<a role="link" aria-disabled="true">Export</a>"#)]
    #[case(r#"<input type="text" aria-invalid="true">"#)]
    #[case(r#"<input type="password" aria-invalid="true">"#)]
    #[case(r#"<my-menu aria-haspopup="menu">x</my-menu>"#)]
    #[case(r#"<div aria-disabled="">x</div>"#)]
    fn supported_uses_are_not_flagged(#[case] html: &str) {
        assert!(
            run(check_deprecated_global_attributes, html).is_empty(),
            "{html}"
        );
    }

    #[test]
    fn deprecated_global_names_the_implicit_role() {
        let findings = run(
            check_deprecated_global_attributes,
            r#"<section aria-disabled="true">...</section>"#,
        );
        assert_eq!(
            findings[0].message,
            "aria-disabled=\"true\" is deprecated on role \"generic\" (implicit on <section>)"
        );
    }
}
