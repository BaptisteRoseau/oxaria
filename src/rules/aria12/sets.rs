//! ARIA-ATTR009: one current item per set, one sorted header per table.
//! Only sets the markup makes obvious are checked: siblings, or the items
//! of one list.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{AriaValue, Parsed, aria_value, is_hidden};
use crate::rules::{CheckOptions, Finding};

use super::tree::{exposed_role, has_exposed_role};

const TABLE_ROLES: &[&str] = &["table", "grid", "treegrid"];

/// Elements sharing a set (or a table) and a value.
struct Group<'a> {
    container: ElementRef<'a>,
    value: String,
    members: Vec<ElementRef<'a>>,
}

pub fn check_multiple_current_or_sorted(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    let current = groups(page, current_value, set_of)
        .into_iter()
        .filter(|group| group.members.len() > 1)
        .map(multiple_current_finding);
    let sorted = groups(page, sorted_value, table_of)
        .into_iter()
        .filter(|group| group.members.len() > 1)
        .map(multiple_sorted_finding);
    current.chain(sorted).collect()
}

fn groups<'a>(
    page: &'a RenderedPage,
    value_of: fn(ElementRef<'a>) -> Option<String>,
    container_of: fn(ElementRef<'a>) -> Option<ElementRef<'a>>,
) -> Vec<Group<'a>> {
    let mut groups: Vec<Group> = Vec::new();
    let members = page
        .all()
        .filter(|el| !is_hidden(*el))
        .filter_map(|el| Some((el, value_of(el)?, container_of(el)?)));
    for (el, value, container) in members {
        match groups
            .iter_mut()
            .find(|group| group.container == container && group.value == value)
        {
            Some(group) => group.members.push(el),
            None => groups.push(Group {
                container,
                value,
                members: vec![el],
            }),
        }
    }
    groups
}

fn current_value(el: ElementRef) -> Option<String> {
    token(el, "aria-current")
}

/// Only one header is sorted at a time, whatever the direction.
fn sorted_value(el: ElementRef) -> Option<String> {
    token(el, "aria-sort").map(|_| String::new())
}

/// The attribute's token, unless it is `false` (or `none` for
/// `aria-sort`), which marks no item.
fn token(el: ElementRef, name: &str) -> Option<String> {
    match aria_value(el, name)? {
        Parsed::Valid(AriaValue::Tokens(tokens)) => tokens
            .first()
            .map(|token| token.to_ascii_lowercase())
            .filter(|token| token != "false" && token != "none"),
        _ => None,
    }
}

/// The element's siblings, or the items of its list when it sits in a
/// list item.
fn set_of(el: ElementRef) -> Option<ElementRef> {
    let parent = el.parent()?;
    match has_exposed_role(parent, &["listitem"]) {
        true => parent.parent(),
        false => Some(parent),
    }
}

fn table_of(el: ElementRef) -> Option<ElementRef> {
    el.ancestors()
        .find(|ancestor| exposed_role(*ancestor).is_some_and(|r| TABLE_ROLES.contains(&r.name)))
}

fn multiple_current_finding(group: Group) -> Finding {
    Finding::error(
        "ARIA-ATTR009",
        format!(
            "{} elements of one set have aria-current=\"{}\"",
            group.members.len(),
            group.value
        ),
    )
    .at(group.members[1])
    .help(format!(
        "keep aria-current=\"{}\" on the one current item only (the first is at {})",
        group.value,
        group.members[0].selector()
    ))
}

fn multiple_sorted_finding(group: Group) -> Finding {
    Finding::error(
        "ARIA-ATTR009",
        format!(
            "{} headers of one table have aria-sort set",
            group.members.len()
        ),
    )
    .at(group.container)
    .help(
        "set aria-sort only on the header the table is currently sorted by, and remove it (or \
         set aria-sort=\"none\") on the others",
    )
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(html: &str) -> Vec<Finding> {
        check_multiple_current_or_sorted(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(
        r#"<nav aria-label="Pagination"><a href="?page=1" aria-current="page">1</a>
              <a href="?page=2" aria-current="page">2</a></nav>"#
    )]
    #[case(
        r#"<ul><li><a href="/a" aria-current="page">A</a></li>
              <li><a href="/b" aria-current="page">B</a></li></ul>"#
    )]
    #[case(r#"<table><tr><th aria-sort="ascending">A</th><th aria-sort="descending">B</th></tr></table>"#)]
    fn several_current_or_sorted_are_flagged(#[case] html: &str) {
        assert_eq!(run(html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(
        r#"<nav aria-label="Pagination"><a href="?page=1">1</a>
              <a href="?page=2" aria-current="page">2</a></nav>"#
    )]
    #[case(r#"<a href="/" aria-current="page">Logo</a><nav><a href="/" aria-current="page">Home</a></nav>"#)]
    #[case(r#"<ol><li><a href="/a" aria-current="step">A</a></li><li><a href="/b" aria-current="false">B</a></li></ol>"#)]
    #[case(r#"<div><a href="/a" aria-current="page">A</a><a href="/b" aria-current="date">B</a></div>"#)]
    #[case(
        r#"<table><tr><th aria-sort="ascending">A</th><th aria-sort="none">B</th></tr></table>"#
    )]
    #[case(
        r#"<table><tr><th aria-sort="ascending">A</th></tr></table>
              <table><tr><th aria-sort="ascending">B</th></tr></table>"#
    )]
    fn single_current_or_sorted_is_not_flagged(#[case] html: &str) {
        assert!(run(html).is_empty(), "{html}");
    }
}
