//! Required context and owned elements: roles inside the container they
//! need (ARIA-STRUCT001), containers owning the items they need
//! (ARIA-STRUCT002), what listbox groups and spinbuttons may contain
//! (ARIA-STRUCT003), and where a `caption` goes (ARIA-STRUCT004).
//!
//! Only explicit roles are checked: native elements get their implicit
//! roles from their context (an `li` outside a list is `generic`).

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{Role, aria_idrefs, first_valid_role, is_aria_true, is_hidden};
use crate::rules::{CheckOptions, Finding};

use super::support::or_list;
use super::tree::{Tree, exposed_role, has_exposed_role};

const CAPTIONED_TABLE_ROLES: &[&str] = &["table", "grid", "treegrid"];
const SPINBUTTON_PARTS: &[(&str, usize)] = &[("textbox", 1), ("button", 2)];

/// The element's explicit role, when it is also the role it's exposed with.
fn explicit_exposed_role(el: ElementRef) -> Option<&'static Role> {
    let role = first_valid_role(el)?;
    exposed_role(el)
        .is_some_and(|exposed| exposed.name == role.name)
        .then_some(role)
}

/// ARIA-STRUCT001: a role with a required context means nothing outside it.
pub fn check_required_context(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let tree = Tree::new(page);
    page.all()
        .filter(|el| !is_hidden(*el))
        .filter_map(|el| {
            let role =
                explicit_exposed_role(el).filter(|role| !role.required_context.is_empty())?;
            let context = tree.context(el);
            (!is_in_required_context(&tree, role, context))
                .then(|| required_context_finding(el, role, context.map(|(_, role)| role.name)))
        })
        .collect()
}

fn is_in_required_context(
    tree: &Tree,
    role: &Role,
    context: Option<(ElementRef, &'static Role)>,
) -> bool {
    match context {
        Some((group, context)) if context.name == "group" && role.name != "treeitem" => {
            role.required_context.contains(&"group") && is_group_in_context(tree, role, group)
        }
        Some((_, context)) => role.required_context.contains(&context.name),
        None => false,
    }
}

/// An `option`'s group must itself be in a `listbox`, a menu item's in a
/// `menu` or `menubar`.
fn is_group_in_context(tree: &Tree, role: &Role, group: ElementRef) -> bool {
    tree.context(group).is_some_and(|(_, context)| {
        context.name != "group" && role.required_context.contains(&context.name)
    })
}

fn required_context_finding(el: ElementRef, role: &Role, actual: Option<&str>) -> Finding {
    let expected = or_list(
        &role
            .required_context
            .iter()
            .copied()
            .filter(|name| *name != "group" || role.name == "treeitem")
            .collect::<Vec<_>>(),
    );
    let actual = match actual {
        Some(actual) => format!(", but its closest ancestor with a role is \"{actual}\""),
        None => ", but no ancestor has a role".to_string(),
    };
    Finding::error(
        "ARIA-STRUCT001",
        format!("role=\"{}\" must be inside {expected}{actual}", role.name),
    )
    .at(el)
    .help(format!(
        "put it inside an element with role {expected}, or reference it from one with aria-owns"
    ))
}

/// ARIA-STRUCT002: a container role that owns none of its required items
/// is an empty widget to assistive technologies. Empty containers pass:
/// script may fill them.
pub fn check_required_owned(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let tree = Tree::new(page);
    page.all()
        .filter(|el| !is_hidden(*el) && !is_busy(*el))
        .filter_map(|el| {
            let role = explicit_exposed_role(el).filter(|role| !role.required_owned.is_empty())?;
            let owned = tree.owned(el);
            let is_missing = !owned
                .iter()
                .any(|item| is_required_item(&tree, role, *item))
                && (!owned.is_empty() || !el.text().is_empty());
            is_missing.then(|| required_owned_finding(el, role, &owned))
        })
        .collect()
}

fn is_busy(el: ElementRef) -> bool {
    std::iter::once(el)
        .chain(el.ancestors())
        .any(|el| is_aria_true(el, "aria-busy"))
}

/// Subclass roles don't count: a `list` needs `listitem`, not `treeitem`.
fn is_required_item(tree: &Tree, role: &Role, item: ElementRef) -> bool {
    let Some(item_role) = exposed_role(item) else {
        return false;
    };
    role.required_owned
        .iter()
        .any(|required| match required.via {
            None => required.role == item_role.name,
            Some(via) => {
                via == item_role.name
                    && tree
                        .owned(item)
                        .iter()
                        .any(|inner| has_exposed_role(*inner, &[required.role]))
            }
        })
}

fn required_owned_finding(el: ElementRef, role: &Role, owned: &[ElementRef]) -> Finding {
    let mut required: Vec<&str> = Vec::new();
    for owned in role.required_owned {
        if !required.contains(&owned.role) {
            required.push(owned.role);
        }
    }
    let required = or_list(&required);
    let actual: Vec<&str> = owned
        .iter()
        .filter_map(|item| exposed_role(*item).map(|role| role.name))
        .collect();
    let actual = match actual.as_slice() {
        [] => "it only holds text or elements without a role".to_string(),
        roles => format!("it owns: {}", roles.join(", ")),
    };
    Finding::error(
        "ARIA-STRUCT002",
        format!(
            "role=\"{}\" owns no element with role {required}",
            role.name
        ),
    )
    .at(el)
    .help(format!(
        "give its items role {required} ({actual}), or set aria-busy=\"true\" while they load"
    ))
}

/// ARIA-STRUCT003: a `group` in a `listbox` holds only options, and a
/// `spinbutton` only its text field and two buttons.
pub fn check_restricted_children(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let tree = Tree::new(page);
    page.all()
        .filter(|el| !is_hidden(*el))
        .flat_map(|el| match exposed_role(el).map(|role| role.name) {
            Some("group") if is_in_listbox(&tree, el) => listbox_group_findings(&tree, el),
            Some("spinbutton") => spinbutton_finding(&tree, el).into_iter().collect(),
            _ => Vec::new(),
        })
        .collect()
}

fn is_in_listbox(tree: &Tree, group: ElementRef) -> bool {
    tree.context(group)
        .is_some_and(|(_, role)| role.name == "listbox")
}

fn listbox_group_findings(tree: &Tree, group: ElementRef) -> Vec<Finding> {
    tree.owned(group)
        .into_iter()
        .filter_map(|item| Some((item, exposed_role(item)?)))
        .filter(|(_, role)| role.name != "option")
        .map(|(item, role)| {
            Finding::error(
                "ARIA-STRUCT003",
                format!(
                    "a group in a listbox contains role \"{}\", but may only contain options",
                    role.name
                ),
            )
            .at(item)
            .help(
                "keep only role=\"option\" items in the group, and name the group with \
                 aria-label or aria-labelledby instead of a heading inside it",
            )
        })
        .collect()
}

fn spinbutton_finding(tree: &Tree, spinbutton: ElementRef) -> Option<Finding> {
    let parts: Vec<&str> = tree
        .owned(spinbutton)
        .into_iter()
        .filter_map(|item| exposed_role(item).map(|role| role.name))
        .collect();
    let fits =
        |(name, limit): &(&str, usize)| parts.iter().filter(|part| *part == name).count() <= *limit;
    let is_valid = parts
        .iter()
        .all(|part| SPINBUTTON_PARTS.iter().any(|(name, _)| name == part))
        && SPINBUTTON_PARTS.iter().all(fits);
    (!is_valid).then(|| {
        Finding::error(
            "ARIA-STRUCT003",
            format!("a spinbutton contains roles {}", parts.join(", ")),
        )
        .at(spinbutton)
        .help(
            "a spinbutton may only contain one textbox and two buttons (increment and \
             decrement); move anything else outside it",
        )
    })
}

/// ARIA-STRUCT004: a `caption` comes first in its table (first or last in
/// its figure), and names it through `aria-labelledby`.
pub fn check_caption_placement(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| {
            !is_hidden(*el) && explicit_exposed_role(*el).is_some_and(|r| r.name == "caption")
        })
        .filter_map(|caption| Some((caption, caption.parent()?)))
        .filter_map(|(caption, parent)| Some((caption, parent, exposed_role(parent)?.name)))
        .flat_map(|(caption, parent, parent_role)| {
            [
                misplaced_caption(caption, parent, parent_role),
                unreferenced_caption(caption, parent, parent_role),
            ]
            .into_iter()
            .flatten()
        })
        .collect()
}

fn element_children<'a>(el: ElementRef<'a>) -> Vec<ElementRef<'a>> {
    el.children()
        .filter(|child| !child.node().is_text() && !child.tag().is_empty())
        .collect()
}

fn misplaced_caption(
    caption: ElementRef,
    parent: ElementRef,
    parent_role: &str,
) -> Option<Finding> {
    let children = element_children(parent);
    let is_first = children.first() == Some(&caption);
    let is_last = children.last() == Some(&caption);
    let (is_placed, place) = match parent_role {
        "figure" => (is_first || is_last, "the first or last child of its figure"),
        role if CAPTIONED_TABLE_ROLES.contains(&role) => (is_first, "the first child of its table"),
        _ => return None,
    };
    (!is_placed).then(|| {
        Finding::error(
            "ARIA-STRUCT004",
            format!("a caption of a {parent_role} isn't {place}"),
        )
        .at(caption)
        .help(format!("move the caption so it is {place}"))
    })
}

fn unreferenced_caption(
    caption: ElementRef,
    parent: ElementRef,
    parent_role: &str,
) -> Option<Finding> {
    let is_referenced = aria_idrefs(parent, "aria-labelledby")
        .into_iter()
        .filter_map(|id| parent.page().element_by_id(id))
        .any(|label| label == caption || label.ancestors().any(|a| a == caption));
    let is_captionable = parent_role == "figure" || CAPTIONED_TABLE_ROLES.contains(&parent_role);
    let id = caption.attr("id").unwrap_or("caption");
    (is_captionable && !is_referenced).then(|| {
        Finding::error(
            "ARIA-STRUCT004",
            format!("the {parent_role} isn't named by its caption"),
        )
        .at(parent)
        .help(format!(
            "give the caption an id and reference it from the {parent_role}, e.g. \
             aria-labelledby=\"{id}\""
        ))
    })
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
    #[case(r#"<div role="tab" aria-selected="true">General</div>"#)]
    #[case(r#"<ul><li role="option" aria-selected="false">Zebra</li></ul>"#)]
    #[case(r#"<div role="group"><div role="option">Zebra</div></div>"#)]
    #[case(r#"<div role="list"><div role="row"><div role="cell">x</div></div></div>"#)]
    fn roles_outside_their_context_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_required_context, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<div role="tablist" aria-label="Settings"><div role="tab" aria-selected="true">General</div></div>"#)]
    #[case(r#"<ul role="listbox" aria-label="Animals"><li role="option" aria-selected="false">Zebra</li></ul>"#)]
    #[case(r#"<ul role="tablist"><li><a role="tab" href="/a">A</a></li></ul>"#)]
    #[case(r#"<div role="listbox"><div role="group"><div role="option">Zebra</div></div></div>"#)]
    #[case(r#"<div role="tree"><div role="treeitem">A<div role="group"><div role="treeitem">B</div></div></div></div>"#)]
    #[case(r#"<div role="menu" aria-owns="m"></div><div id="m" role="menuitem">Open</div>"#)]
    #[case(r#"<table><tr><td role="cell">x</td></tr></table>"#)]
    #[case(r#"<div role="tab" hidden>General</div>"#)]
    fn roles_in_their_context_are_not_flagged(#[case] html: &str) {
        assert!(run(check_required_context, html).is_empty(), "{html}");
    }

    #[test]
    fn required_context_names_both_roles() {
        let findings = run(
            check_required_context,
            r#"<ul><li role="option">Zebra</li></ul>"#,
        );
        assert_eq!(
            findings[0].message,
            "role=\"option\" must be inside listbox, but its closest ancestor with a role is \"list\""
        );
    }

    #[rstest]
    #[case(r#"<div role="tablist" aria-label="Settings"><button>General</button><button>Privacy</button></div>"#)]
    #[case(r#"<div role="list"><div>Just text</div></div>"#)]
    #[case(r#"<div role="list"><div role="treeitem">Not a listitem</div></div>"#)]
    fn containers_without_their_items_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_required_owned, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<div role="tablist" aria-label="Settings"><button role="tab" aria-selected="true">General</button></div>"#)]
    #[case(r#"<div role="list"></div>"#)]
    #[case(r#"<div role="list" aria-busy="true"><div>Loading</div></div>"#)]
    #[case(r#"<ul role="list"><li>A</li></ul>"#)]
    #[case(r#"<div role="listbox"><div role="group"><div role="option">A</div></div></div>"#)]
    #[case(r#"<div role="grid"><div role="rowgroup"><div role="row"><div role="gridcell">A</div></div></div></div>"#)]
    #[case(r#"<ul role="menu"><li><a role="menuitem" href="/a">A</a></li></ul>"#)]
    #[case("<ul><li>native lists aren't checked</li></ul>")]
    fn containers_with_their_items_are_not_flagged(#[case] html: &str) {
        assert!(run(check_required_owned, html).is_empty(), "{html}");
    }

    #[test]
    fn heading_in_listbox_group_is_flagged() {
        let html = r#"<ul role="listbox" aria-label="Fruit"><li role="group" aria-label="Citrus">
            <span role="heading" aria-level="3">Citrus</span>
            <span role="option" aria-selected="false">Lemon</span></li></ul>"#;
        assert_eq!(run(check_restricted_children, html).len(), 1);
    }

    #[rstest]
    #[case(
        r#"<ul role="listbox" aria-label="Fruit"><li role="group" aria-label="Citrus">
            <span role="option" aria-selected="false">Lemon</span>
            <span role="option" aria-selected="false">Lime</span></li></ul>"#
    )]
    #[case(r#"<div role="spinbutton" aria-valuenow="1"><input type="text"><button>-</button><button>+</button></div>"#)]
    #[case(r#"<div role="group"><h3>Not in a listbox</h3></div>"#)]
    fn allowed_children_are_not_flagged(#[case] html: &str) {
        assert!(run(check_restricted_children, html).is_empty(), "{html}");
    }

    #[test]
    fn spinbutton_with_extra_parts_is_flagged() {
        let html = r#"<div role="spinbutton"><input><button>-</button><button>+</button><button>x</button></div>"#;
        assert_eq!(run(check_restricted_children, html).len(), 1);
    }

    #[test]
    fn misplaced_unreferenced_caption_is_flagged() {
        let html = r#"<div role="table"><div role="rowgroup">x</div><div role="caption">Contest Entrants</div></div>"#;
        let findings = run(check_caption_placement, html);
        assert_eq!(findings.len(), 2, "{findings:?}");
    }

    #[rstest]
    #[case(
        r#"<div role="table" aria-labelledby="name" aria-describedby="desc">
        <div role="caption"><div id="name">Contest Entrants</div><div id="desc">Totals.</div></div>
        <div role="rowgroup">x</div></div>"#
    )]
    #[case(r#"<div role="figure" aria-labelledby="c"><img src="a.png" alt="A"><div role="caption" id="c">A</div></div>"#)]
    #[case("<table><caption>Native</caption><tr><td>1</td></tr></table>")]
    fn placed_and_referenced_caption_is_not_flagged(#[case] html: &str) {
        assert!(run(check_caption_placement, html).is_empty(), "{html}");
    }
}
