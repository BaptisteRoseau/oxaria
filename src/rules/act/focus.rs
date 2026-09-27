//! ACT rule 307n5z for focusable content hidden by presentational children.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{effective_role, is_tabbable};
use crate::rules::{CheckOptions, Finding};

/// 307n5z: an element whose role makes its children presentational (a
/// button, checkbox, tab, image, ...) must not contain anything in the
/// sequential focus order: assistive technologies never reach it.
pub fn check_focusable_in_presentational_children(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.all()
        .filter_map(|el| Some((el, effective_role(el)?)))
        .filter(|(_, role)| role.children_presentational)
        .filter_map(|(el, role)| {
            let focusable = el
                .descendants()
                .find(|descendant| is_tabbable(*descendant))?;
            Some(
                Finding::error(
                    "307n5z",
                    format!(
                        "<{}> (role {}) contains focusable {}, which its role hides from \
                         assistive technologies",
                        el.tag(),
                        role.name,
                        markup(focusable)
                    ),
                )
                .at(el)
                .help(format!(
                    "move the <{}> out of the <{}> (e.g. next to it), or take it out of the \
                     tab order",
                    focusable.tag(),
                    el.tag()
                )),
            )
        })
        .collect()
}

fn markup(el: ElementRef) -> String {
    match (el.attr("href"), el.attr("tabindex")) {
        (Some(href), _) => format!("<{} href=\"{href}\">", el.tag()),
        (None, Some(tabindex)) => format!("<{} tabindex=\"{tabindex}\">", el.tag()),
        (None, None) => format!("<{}>", el.tag()),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn count(html: &str) -> usize {
        check_focusable_in_presentational_children(&page_from_html(html), &CheckOptions::default())
            .len()
    }

    #[rstest]
    #[case(
        r#"<button>Save <span role="button" aria-label="options" tabindex="0">v</span></button>"#
    )]
    #[case(r#"<p role="checkbox" aria-checked="false" tabindex="0">I agree to the <a href="/terms">terms</a></p>"#)]
    #[case(r#"<ul role="menu"><li role="menuitemcheckbox" aria-checked="true"><input type="checkbox" checked> Sort</li></ul>"#)]
    #[case(r##"<ul role="tablist"><li role="tab"><a href="#">Tab 1</a></li></ul>"##)]
    #[case(r##"<span role="img" aria-label="ASCII art">*** <a href="#">a link</a></span>"##)]
    fn focusable_descendants_fail(#[case] html: &str) {
        assert_eq!(count(html), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<button>Save</button><button aria-label="options" aria-expanded="false">v</button>"#)]
    #[case(r#"<p id="t"><span role="checkbox" aria-checked="false" tabindex="0" aria-labelledby="t">I agree</span> <a href="/terms">terms</a></p>"#)]
    #[case(r#"<ul role="menu"><li role="menuitemcheckbox" aria-checked="true"><input type="checkbox" role="none" disabled checked> Sort</li></ul>"#)]
    #[case("<button><a>button/link</a></button>")]
    #[case(r#"<button><span tabindex="-1">x</span></button>"#)]
    #[case(r##"<table><tr><th><a href="#">header link</a></th></tr><tr><td><a href="#">cell link</a></td></tr></table>"##)]
    #[case(r#"<a href="https://w3.org"><span tabindex="0">W3C</span></a>"#)]
    fn unfocusable_descendants_or_other_roles_pass(#[case] html: &str) {
        assert_eq!(count(html), 0, "{html}");
    }
}
