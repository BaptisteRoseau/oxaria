//! ID references: that they resolve (ARIA-IDREF001), single, acyclic
//! `aria-owns` ownership (ARIA-IDREF002) and `aria-activedescendant`
//! pointing inside its widget (ARIA-IDREF003).

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    AriaValue, Parsed, aria_idrefs, aria_value, first_valid_role, is_hidden,
};
use crate::rules::{CheckOptions, Finding};

use super::support::shortened;
use super::tree::{Tree, exposed_role};

const IDREF_ATTRIBUTES: &[&str] = &[
    "aria-activedescendant",
    "aria-controls",
    "aria-describedby",
    "aria-details",
    "aria-errormessage",
    "aria-flowto",
    "aria-labelledby",
    "aria-owns",
];

/// Roles whose `aria-activedescendant` may point into the popup they
/// control rather than their own descendants.
const CONTROLLING_ROLES: &[&str] = &["combobox", "searchbox", "textbox"];

/// ARIA-IDREF001: a reference to a missing or duplicated id relates the
/// element to nothing, or to an arbitrary element.
pub fn check_unresolved_reference(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let tree = Tree::new(page);
    page.all()
        .flat_map(|el| {
            IDREF_ATTRIBUTES
                .iter()
                .filter(move |name| !is_popup_not_rendered_yet(el, name))
                .flat_map(move |name| aria_idrefs(el, name).into_iter().map(move |id| (name, id)))
                .filter_map(|(name, id)| unresolved_reference(&tree, el, name, id))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// A collapsed control's popup is often only created when it opens.
fn is_popup_not_rendered_yet(el: ElementRef, name: &str) -> bool {
    name == "aria-controls"
        && aria_value(el, "aria-expanded") == Some(Parsed::Valid(AriaValue::False))
}

fn unresolved_reference(tree: &Tree, el: ElementRef, name: &str, id: &str) -> Option<Finding> {
    let count = tree.id_count(id);
    let id = shortened(id);
    let (message, help) = match count {
        0 => (
            format!("{name} references id \"{id}\", which does not exist in the document"),
            format!(
                "add id=\"{id}\" to the element it refers to, or fix or remove the reference in {name}"
            ),
        ),
        1 => return None,
        count => (
            format!("{name} references id \"{id}\", which {count} elements share"),
            format!("give each element its own id, and point {name} at the right one"),
        ),
    };
    Some(Finding::error("ARIA-IDREF001", message).at(el).help(help))
}

/// ARIA-IDREF002: an element has one owner, and ownership can't loop.
pub fn check_aria_owns_conflict(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let tree = Tree::new(page);
    let owns: Vec<(ElementRef, &str)> = page
        .all()
        .flat_map(|el| {
            aria_idrefs(el, "aria-owns")
                .into_iter()
                .map(move |id| (el, id))
        })
        .filter(|(_, id)| tree.by_id(id).is_some())
        .collect();
    owns.iter()
        .enumerate()
        .filter_map(|(index, (owner, id))| {
            let first_owner = owns[..index]
                .iter()
                .find(|(other, other_id)| other_id == id && other != owner)
                .map(|(other, _)| *other);
            match first_owner {
                Some(first) => Some(second_owner_finding(*owner, id, first)),
                None => circular_ownership_finding(&tree, *owner, id),
            }
        })
        .collect()
}

fn second_owner_finding(owner: ElementRef, id: &str, first: ElementRef) -> Finding {
    Finding::error(
        "ARIA-IDREF002",
        format!("id \"{id}\" is already in the aria-owns of another element"),
    )
    .at(owner)
    .help(format!(
        "keep \"{id}\" in only one element's aria-owns (it is also owned by {})",
        first.selector()
    ))
}

fn circular_ownership_finding(tree: &Tree, owner: ElementRef, id: &str) -> Option<Finding> {
    let owned = tree.by_id(id)?;
    let is_circular = tree
        .self_and_ancestors(owner)
        .skip(1)
        .chain([owner])
        .any(|ancestor| ancestor == owned);
    is_circular.then(|| {
        Finding::error(
            "ARIA-IDREF002",
            format!("aria-owns=\"{id}\" makes the element own itself or one of its owners"),
        )
        .at(owner)
        .help(format!(
            "remove \"{id}\" from aria-owns: an element can't own one of its own ancestors"
        ))
    })
}

/// ARIA-IDREF003: the active descendant must be inside the widget that
/// has focus, or inside the popup a text field controls.
pub fn check_active_descendant_ownership(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    let tree = Tree::new(page);
    page.all()
        .filter(|el| !is_hidden(*el))
        .filter_map(|el| {
            let id = *aria_idrefs(el, "aria-activedescendant").first()?;
            let target = tree.by_id(id)?;
            (!is_owned_by(&tree, target, el) && !is_in_controlled_popup(&tree, target, el))
                .then(|| active_descendant_finding(el, id))
        })
        .collect()
}

fn is_owned_by(tree: &Tree, target: ElementRef, el: ElementRef) -> bool {
    tree.self_and_ancestors(target)
        .skip(1)
        .any(|ancestor| ancestor == el)
}

fn is_in_controlled_popup(tree: &Tree, target: ElementRef, el: ElementRef) -> bool {
    let controls = exposed_role(el)
        .or_else(|| first_valid_role(el))
        .is_some_and(|role| CONTROLLING_ROLES.contains(&role.name));
    controls
        && aria_idrefs(el, "aria-controls")
            .into_iter()
            .filter_map(|id| tree.by_id(id))
            .any(|popup| is_owned_by(tree, target, popup))
}

fn active_descendant_finding(el: ElementRef, id: &str) -> Finding {
    Finding::error(
        "ARIA-IDREF003",
        format!("aria-activedescendant=\"{id}\" points at an element outside this widget"),
    )
    .at(el)
    .help(format!(
        "move #{id} inside the element, own it with aria-owns=\"{id}\", or (on a combobox or \
         text field) reference its popup with aria-controls"
    ))
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
    #[case(r#"<input type="text" aria-labelledby="startTime-label">"#, 1)]
    #[case(
        r#"<button aria-controls="panel-1 panel-2">Expand all</button><div id="panel-1">x</div>"#,
        1
    )]
    #[case(r#"<input aria-errormessage="e"><p id="e">x</p><p id="e">y</p>"#, 1)]
    #[case(
        r#"<div role="listbox" aria-activedescendant="gone" tabindex="0"></div>"#,
        1
    )]
    fn unresolved_references_are_flagged(#[case] html: &str, #[case] count: usize) {
        assert_eq!(run(check_unresolved_reference, html).len(), count, "{html}");
    }

    #[rstest]
    #[case(r#"<span id="startTime-label">Start time</span><input type="text" aria-labelledby="startTime-label">"#)]
    #[case(r#"<button aria-controls="panel-1 panel-2">Expand all</button><div id="panel-1">x</div><div id="panel-2">y</div>"#)]
    #[case(r#"<button aria-expanded="false" aria-controls="menu">Menu</button>"#)]
    #[case(r#"<input aria-labelledby="">"#)]
    fn resolved_references_are_not_flagged(#[case] html: &str) {
        assert!(run(check_unresolved_reference, html).is_empty(), "{html}");
    }

    #[test]
    fn missing_id_help_names_it() {
        let findings = run(
            check_unresolved_reference,
            r#"<input aria-describedby="hint">"#,
        );
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with("add id=\"hint\"")
        );
    }

    #[rstest]
    #[case(
        r#"<div role="tree" aria-owns="node-7">x</div><div role="group" aria-owns="node-7">y</div>
              <div role="treeitem" id="node-7" aria-selected="false">Reports</div>"#
    )]
    #[case(
        r#"<div id="a" role="group" aria-owns="b"><div id="b" role="group">x</div></div>
              <div id="c" aria-owns="c">x</div>"#
    )]
    #[case(r#"<div id="outer" role="tree"><div role="group" aria-owns="outer">x</div></div>"#)]
    fn ownership_conflicts_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_aria_owns_conflict, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(
        r#"<div role="tree" aria-label="Files"><div role="group" aria-owns="node-7"></div></div>
              <div role="treeitem" id="node-7" aria-selected="false">Reports</div>"#
    )]
    #[case(r#"<div role="list" aria-owns="missing"></div>"#)]
    fn single_ownership_is_not_flagged(#[case] html: &str) {
        assert!(run(check_aria_owns_conflict, html).is_empty(), "{html}");
    }

    #[test]
    fn active_descendant_outside_the_widget_is_flagged() {
        let html = r#"<ul role="listbox" tabindex="0" aria-label="Tags" aria-activedescendant="opt-zoom">
              <li role="option" aria-selected="false">Zebra</li></ul>
            <li role="option" id="opt-zoom" aria-selected="false">Zoom</li>"#;
        assert_eq!(run(check_active_descendant_ownership, html).len(), 1);
    }

    #[rstest]
    #[case(
        r#"<label for="tag_combo">Tag</label>
        <input type="text" id="tag_combo" role="combobox" aria-autocomplete="list"
          aria-haspopup="listbox" aria-expanded="true"
          aria-controls="popup_listbox" aria-activedescendant="selected_option">
        <ul role="listbox" id="popup_listbox">
          <li role="option" aria-selected="false">Zebra</li>
          <li role="option" id="selected_option" aria-selected="true">Zoom</li></ul>"#
    )]
    #[case(r#"<ul role="listbox" tabindex="0" aria-activedescendant="o1"><li role="option" id="o1">A</li></ul>"#)]
    #[case(r#"<div role="listbox" tabindex="0" aria-owns="o1" aria-activedescendant="o1"></div><div role="option" id="o1">A</div>"#)]
    fn active_descendant_inside_the_widget_is_not_flagged(#[case] html: &str) {
        assert!(
            run(check_active_descendant_ownership, html).is_empty(),
            "{html}"
        );
    }
}
