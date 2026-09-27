//! The accessibility tree's ownership, as far as markup gives it: DOM
//! nesting, overridden by `aria-owns`, with elements that expose no role
//! of their own (`generic`, `none`, no role at all) left out.

use std::collections::HashMap;

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{Role, aria_idrefs, effective_role, is_hidden};

pub struct Tree<'a> {
    ids: HashMap<&'a str, Vec<ElementRef<'a>>>,
    /// id -> the first element whose `aria-owns` lists it.
    owners: HashMap<&'a str, ElementRef<'a>>,
    /// Bounds walks up the tree, which `aria-owns` cycles would make endless.
    size: usize,
}

impl<'a> Tree<'a> {
    pub fn new(page: &'a RenderedPage) -> Self {
        let mut ids: HashMap<&str, Vec<ElementRef>> = HashMap::new();
        let mut owners = HashMap::new();
        for el in page.all() {
            if let Some(id) = el.attr("id") {
                ids.entry(id).or_default().push(el);
            }
            for id in aria_idrefs(el, "aria-owns") {
                owners.entry(id).or_insert(el);
            }
        }
        Tree {
            ids,
            owners,
            size: page.elements.len(),
        }
    }

    /// The first element with this id, as `getElementById` returns.
    pub fn by_id(&self, id: &str) -> Option<ElementRef<'a>> {
        self.ids
            .get(id)
            .and_then(|elements| elements.first().copied())
    }

    pub fn id_count(&self, id: &str) -> usize {
        self.ids.get(id).map_or(0, Vec::len)
    }

    /// The `aria-owns` owner if there is one, else the DOM parent.
    pub fn parent(&self, el: ElementRef<'a>) -> Option<ElementRef<'a>> {
        self.owner(el).or_else(|| el.parent())
    }

    /// The element and its ancestors through [`Tree::parent`].
    pub fn self_and_ancestors(&self, el: ElementRef<'a>) -> impl Iterator<Item = ElementRef<'a>> {
        std::iter::successors(Some(el), |el| self.parent(*el)).take(self.size)
    }

    /// The nearest ancestor exposing a role of its own.
    pub fn context(&self, el: ElementRef<'a>) -> Option<(ElementRef<'a>, &'static Role)> {
        self.self_and_ancestors(el)
            .skip(1)
            .find_map(|ancestor| exposed_role(ancestor).map(|role| (ancestor, role)))
    }

    /// The owned elements exposing a role, looking through those that
    /// don't; hidden ones are left out.
    pub fn owned(&self, el: ElementRef<'a>) -> Vec<ElementRef<'a>> {
        let mut owned = Vec::new();
        self.collect_owned(el, &mut owned);
        let targets = aria_idrefs(el, "aria-owns")
            .into_iter()
            .filter_map(|id| self.by_id(id))
            .filter(|target| *target != el && !is_hidden(*target));
        for target in targets {
            match exposed_role(target) {
                Some(_) => owned.push(target),
                None => self.collect_owned(target, &mut owned),
            }
        }
        owned
    }

    fn collect_owned(&self, el: ElementRef<'a>, owned: &mut Vec<ElementRef<'a>>) {
        let children = el
            .children()
            .filter(|child| !child.node().is_text() && !is_hidden(*child))
            .filter(|child| self.owner(*child).is_none());
        for child in children {
            match exposed_role(child) {
                Some(_) => owned.push(child),
                None => self.collect_owned(child, owned),
            }
        }
    }

    fn owner(&self, el: ElementRef<'a>) -> Option<ElementRef<'a>> {
        el.attr("id")
            .and_then(|id| self.owners.get(id))
            .copied()
            .filter(|owner| *owner != el)
    }
}

/// The element's role, unless it is `generic` or presentational, which
/// the accessibility tree leaves out.
pub fn exposed_role(el: ElementRef) -> Option<&'static Role> {
    effective_role(el).filter(|role| role.name != "generic" && !role.is_presentational())
}

pub fn has_exposed_role(el: ElementRef, names: &[&str]) -> bool {
    exposed_role(el).is_some_and(|role| names.contains(&role.name))
}

pub fn presentational_children_role(el: ElementRef) -> Option<&'static Role> {
    exposed_role(el).filter(|role| role.children_presentational)
}

/// Inside a role whose children are presentational, the element's own
/// role isn't exposed.
pub fn has_presentational_children_ancestor(el: ElementRef) -> bool {
    el.ancestors()
        .any(|ancestor| presentational_children_role(ancestor).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn owned_tags(html: &str, id: &str) -> Vec<String> {
        let page = page_from_html(html);
        let tree = Tree::new(&page);
        let el = tree.by_id(id).unwrap();
        tree.owned(el)
            .iter()
            .map(|el| el.tag().to_string())
            .collect()
    }

    #[test]
    fn owned_elements_look_through_generic_ones() {
        let html = r#"<ul id="l" role="listbox"><div><li role="option">a</li></div><b>x</b></ul>"#;
        assert_eq!(owned_tags(html, "l"), ["li"]);
    }

    #[test]
    fn aria_owns_moves_elements() {
        let html = r#"<div id="a" role="list" aria-owns="i"></div>
                      <div id="b" role="list"><p id="i" role="listitem">x</p></div>"#;
        assert_eq!(owned_tags(html, "a"), ["p"]);
        assert!(owned_tags(html, "b").is_empty());
    }

    #[test]
    fn context_skips_elements_without_a_role() {
        let page =
            page_from_html(r#"<div role="tablist"><span><b id="t" role="tab">x</b></span></div>"#);
        let tree = Tree::new(&page);
        let (_, role) = tree.context(tree.by_id("t").unwrap()).unwrap();
        assert_eq!(role.name, "tablist");
    }

    #[test]
    fn ownership_cycles_end() {
        let page =
            page_from_html(r#"<div id="a" aria-owns="b"><div id="b" aria-owns="a"></div></div>"#);
        let tree = Tree::new(&page);
        assert!(tree.context(tree.by_id("a").unwrap()).is_none());
    }
}
