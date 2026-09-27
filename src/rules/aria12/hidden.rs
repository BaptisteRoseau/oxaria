//! ARIA-HIDDEN001: content hidden with `aria-hidden` must not be
//! focusable, and `aria-hidden="false"` can't reveal part of it.

use std::collections::HashSet;

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    AriaValue, Parsed, aria_idrefs, aria_value, is_aria_true, is_tabbable,
};
use crate::rules::{CheckOptions, Finding};

/// Only elements in the Tab order: `tabindex="-1"` is the usual way to
/// take a hidden duplicate (e.g. a card's image link) out of it.
pub fn check_hidden_content(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let is_measured = page.all().any(|el| !is_unmeasured(el));
    let collapsed = collapsed_ids(page);
    page.all()
        .filter_map(|el| {
            let hidden_by = hiding_element(el)?;
            let is_undisplayed =
                (is_measured && is_unmeasured(hidden_by)) || is_collapsed(hidden_by, &collapsed);
            match is_undisplayed {
                true => None,
                false => focusable_hidden_finding(el, hidden_by)
                    .or_else(|| revealed_hidden_finding(el, hidden_by)),
            }
        })
        .collect()
}

/// A 0x0 box means "not laid out": the hidden subtree is usually also
/// `display: none` (a closed menu toggled by a class), so nothing in it is
/// focusable. Pages that were never measured (test fixtures) are all 0x0.
fn is_unmeasured(el: ElementRef) -> bool {
    let bbox = el.bounding_box();
    bbox.width == 0.0 && bbox.height == 0.0
}

/// Ids controlled by a collapsed control (`aria-expanded="false"`).
fn collapsed_ids(page: &RenderedPage) -> HashSet<&str> {
    page.all()
        .filter(|el| aria_value(*el, "aria-expanded") == Some(Parsed::Valid(AriaValue::False)))
        .flat_map(|el| aria_idrefs(el, "aria-controls"))
        .collect()
}

/// A collapsed accordion panel or menu is hidden with `aria-hidden` and,
/// in an external stylesheet the render doesn't load, `display: none`.
fn is_collapsed(hidden_by: ElementRef, collapsed: &HashSet<&str>) -> bool {
    std::iter::once(hidden_by)
        .chain(hidden_by.ancestors())
        .filter_map(|el| el.attr("id"))
        .any(|id| collapsed.contains(id))
}

/// The nearest element, itself included, with `aria-hidden="true"`.
fn hiding_element(el: ElementRef) -> Option<ElementRef> {
    std::iter::once(el)
        .chain(el.ancestors())
        .find(|el| is_aria_true(*el, "aria-hidden"))
}

fn focusable_hidden_finding(el: ElementRef, hidden_by: ElementRef) -> Option<Finding> {
    is_tabbable(el).then(|| {
        let location = match hidden_by == el {
            true => "has aria-hidden=\"true\"".to_string(),
            false => format!(
                "is inside aria-hidden=\"true\" (on {})",
                hidden_by.selector()
            ),
        };
        Finding::error(
            "ARIA-HIDDEN001",
            format!("focusable <{}> {location}", el.tag()),
        )
        .at(el)
        .help(
            "remove aria-hidden so keyboard users don't land on content screen readers can't \
             see, or take the element out of the Tab order with tabindex=\"-1\" (or disable it)",
        )
    })
}

fn revealed_hidden_finding(el: ElementRef, hidden_by: ElementRef) -> Option<Finding> {
    let is_revealed = aria_value(el, "aria-hidden") == Some(Parsed::Valid(AriaValue::False));
    (is_revealed && hidden_by != el).then(|| {
        Finding::error(
            "ARIA-HIDDEN001",
            format!(
                "aria-hidden=\"false\" on <{}> can't reveal it inside aria-hidden=\"true\" (on {})",
                el.tag(),
                hidden_by.selector()
            ),
        )
        .at(el)
        .help("move the element out of the hidden subtree, or remove aria-hidden from its ancestor")
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(html: &str) -> Vec<Finding> {
        check_hidden_content(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(r#"<div aria-hidden="true"><a href="/checkout">Go to checkout</a></div>"#)]
    #[case(r#"<button aria-hidden="true">Close</button>"#)]
    #[case(r#"<div aria-hidden="true"><p aria-hidden="false">Revealed?</p></div>"#)]
    #[case(r#"<div aria-hidden="true"><div tabindex="0">x</div></div>"#)]
    #[case(r#"<div aria-hidden="true" style="width: 800px; height: 20px"><button>Laid out</button></div>"#)]
    fn hidden_focusable_or_revealed_content_is_flagged(#[case] html: &str) {
        assert_eq!(run(html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<a href="/checkout"><svg aria-hidden="true"></svg> Go to checkout</a>"#)]
    #[case(
        r#"<div aria-hidden="true"><a href="/p" tabindex="-1"><img src="p.png" alt=""></a></div>"#
    )]
    #[case(r#"<div aria-hidden="true"><button disabled>x</button></div>"#)]
    #[case(r#"<div aria-hidden="true" hidden><a href="/x">x</a></div>"#)]
    #[case(r#"<p aria-hidden="false">Not in a hidden subtree</p>"#)]
    #[case(r#"<main style="width: 800px; height: 600px"><div aria-hidden="true"><button>Closed menu</button></div></main>"#)]
    #[case(r#"<button aria-expanded="false" aria-controls="panel">FAQ</button>
              <section id="panel" aria-hidden="true"><p><a href="/pricing">Pricing</a></p></section>"#)]
    fn hidden_static_content_is_not_flagged(#[case] html: &str) {
        assert!(run(html).is_empty(), "{html}");
    }
}
