//! ARIA in HTML's requirement against deprecated roles and attributes.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{DEPRECATED_ROLES, aria_attributes, attribute, explicit_roles};
use crate::rules::{CheckOptions, Finding};

use super::markup::start_tag;

/// HTMLARIA015: deprecated `role` tokens and `aria-*` attributes.
pub fn check_deprecated_aria(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| deprecated_roles(el).chain(deprecated_attributes(el)))
        .collect()
}

fn deprecated_roles(el: ElementRef<'_>) -> impl Iterator<Item = Finding> + '_ {
    explicit_roles(el)
        .into_iter()
        .filter(|token| DEPRECATED_ROLES.contains(&token.as_str()))
        .map(move |role| {
            Finding::error(
                "HTMLARIA015",
                format!(
                    "{} uses the deprecated role \"{role}\"",
                    start_tag(el, &["role"])
                ),
            )
            .at(el)
            .help(deprecated_role_help(&role))
        })
}

fn deprecated_role_help(role: &str) -> String {
    match role {
        "directory" => "remove role=\"directory\" and use a native <ul> or <ol>, or \
                        role=\"list\""
            .to_string(),
        role => format!("remove role=\"{role}\" and use a plain <li>"),
    }
}

fn deprecated_attributes(el: ElementRef<'_>) -> impl Iterator<Item = Finding> + '_ {
    aria_attributes(el)
        .filter(|(name, _)| attribute(name).is_some_and(|attribute| attribute.is_deprecated))
        .map(move |(name, _)| {
            Finding::error(
                "HTMLARIA015",
                format!(
                    "{} uses the deprecated attribute {name}",
                    start_tag(el, &[name])
                ),
            )
            .at(el)
            .help(format!(
                "remove {name}: it is deprecated and has no replacement"
            ))
        })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(html: &str) -> Vec<Finding> {
        check_deprecated_aria(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(r#"<ul role="directory"><li>a</li></ul>"#, "use a native <ul>")]
    #[case(r#"<ol><li role="doc-endnote">a</li></ol>"#, "use a plain <li>")]
    #[case(r#"<li role="DOC-BIBLIOENTRY">a</li>"#, "use a plain <li>")]
    #[case(r#"<div aria-grabbed="false">x</div>"#, "no replacement")]
    #[case(r#"<div aria-dropeffect="move">x</div>"#, "no replacement")]
    fn deprecated_features_are_flagged(#[case] html: &str, #[case] help: &str) {
        let findings = run(html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA015");
        assert!(findings[0].help.as_deref().unwrap().contains(help));
    }

    #[rstest]
    #[case("<ul><li>a</li></ul>")]
    #[case("<ol><li>a</li></ol>")]
    #[case(r#"<ul role="list"><li>a</li></ul>"#)]
    #[case(r#"<section role="doc-endnotes">x</section>"#)]
    fn current_features_are_not_flagged(#[case] html: &str) {
        assert!(run(html).is_empty());
    }
}
