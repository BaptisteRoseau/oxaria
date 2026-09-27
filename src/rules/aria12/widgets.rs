//! Widget patterns: the ARIA 1.2 combobox (ARIA-WIDGET001), popups
//! matching `aria-haspopup` (ARIA-WIDGET002), list autocomplete wiring
//! (ARIA-WIDGET003), keyboard-reachable popup triggers (ARIA-WIDGET006)
//! and focusable feed articles (ARIA-WIDGET007).

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    AriaValue, Parsed, aria_idrefs, aria_value, first_valid_role, is_focusable, is_hidden,
};
use crate::rules::{CheckOptions, Finding};

use super::support::{has_value, is_aria_disabled};
use super::tree::{Tree, exposed_role, has_exposed_role};

const COMBOBOX_POPUP_ROLES: &[&str] = &["listbox", "tree", "grid", "dialog"];
const POPUP_ROLES: &[&str] = &["menu", "listbox", "tree", "grid", "dialog", "alertdialog"];
const TEXT_FIELD_ROLES: &[&str] = &["textbox", "searchbox"];

fn token(el: ElementRef, name: &str) -> Option<String> {
    match aria_value(el, name)? {
        Parsed::Valid(AriaValue::Tokens(tokens)) => {
            tokens.first().map(|token| token.to_ascii_lowercase())
        }
        _ => None,
    }
}

/// The `aria-haspopup` value naming a popup, `true` meaning `menu`;
/// `None` when absent or `false`.
fn popup_kind(el: ElementRef) -> Option<String> {
    match token(el, "aria-haspopup")?.as_str() {
        "false" => None,
        "true" => Some("menu".to_string()),
        kind => Some(kind.to_string()),
    }
}

/// `aria-haspopup`'s name for a popup role.
fn popup_kind_of_role(role: &str) -> &str {
    match role {
        "alertdialog" => "dialog",
        role => role,
    }
}

/// The first existing element `aria-controls` references.
fn controlled<'a>(tree: &Tree<'a>, el: ElementRef<'a>) -> Option<ElementRef<'a>> {
    aria_idrefs(el, "aria-controls")
        .into_iter()
        .find_map(|id| tree.by_id(id))
}

/// ARIA-WIDGET001: since ARIA 1.2 the combobox is the text field itself,
/// controls its popup, and always tells whether it's expanded.
pub fn check_combobox_pattern(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let tree = Tree::new(page);
    page.all()
        .filter(|el| !is_hidden(*el) && first_valid_role(*el).is_some_and(|r| r.name == "combobox"))
        .flat_map(|combobox| {
            [
                wrapped_text_field(combobox),
                owns_popup(combobox),
                missing_expanded(combobox),
                wrong_popup_role(&tree, combobox),
                missing_popup_kind(&tree, combobox),
            ]
            .into_iter()
            .flatten()
            .map(move |(message, help)| {
                Finding::error("ARIA-WIDGET001", message)
                    .at(combobox)
                    .help(help)
            })
        })
        .collect()
}

type Problem = Option<(String, String)>;

fn wrapped_text_field(combobox: ElementRef) -> Problem {
    let is_text_field = |el: ElementRef| {
        matches!(el.tag(), "input" | "textarea") || has_exposed_role(el, TEXT_FIELD_ROLES)
    };
    let wraps = !is_text_field(combobox) && combobox.descendants().any(is_text_field);
    wraps.then(|| {
        (
            "role=\"combobox\" wraps a text field (the ARIA 1.1 pattern)".to_string(),
            "put role=\"combobox\" on the <input> itself, and remove it from the wrapper"
                .to_string(),
        )
    })
}

fn owns_popup(combobox: ElementRef) -> Problem {
    has_value(combobox, "aria-owns").then(|| {
        let ids = combobox.attr("aria-owns").unwrap_or_default().trim();
        (
            format!("combobox references its popup with aria-owns=\"{ids}\""),
            format!("use aria-controls=\"{ids}\" instead of aria-owns"),
        )
    })
}

fn missing_expanded(combobox: ElementRef) -> Problem {
    (!has_value(combobox, "aria-expanded")).then(|| {
        (
            "combobox has no aria-expanded".to_string(),
            "set aria-expanded=\"false\", and switch it to \"true\" while the popup is shown"
                .to_string(),
        )
    })
}

fn wrong_popup_role(tree: &Tree, combobox: ElementRef) -> Problem {
    let popup = controlled(tree, combobox)?;
    let role = exposed_role(popup).filter(|role| !COMBOBOX_POPUP_ROLES.contains(&role.name))?;
    Some((
        format!("combobox popup has role \"{}\"", role.name),
        "give the popup role listbox, tree, grid or dialog".to_string(),
    ))
}

fn missing_popup_kind(tree: &Tree, combobox: ElementRef) -> Problem {
    let popup = controlled(tree, combobox)?;
    let role = exposed_role(popup)
        .map(|role| popup_kind_of_role(role.name))
        .filter(|kind| *kind != "listbox" && COMBOBOX_POPUP_ROLES.contains(kind))?;
    (!has_value(combobox, "aria-haspopup")).then(|| {
        (
            format!("combobox opens a {role} but has no aria-haspopup"),
            format!("set aria-haspopup=\"{role}\" (a combobox's popup is a listbox by default)"),
        )
    })
}

/// ARIA-WIDGET002: `aria-haspopup` tells what opens, so it must match the
/// popup's role.
pub fn check_popup_role_match(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let tree = Tree::new(page);
    page.all()
        .filter_map(|trigger| {
            let kind = popup_kind(trigger)?;
            let popup_role = popup_role(controlled(&tree, trigger)?)?;
            let expected = popup_kind_of_role(popup_role);
            (kind != expected).then(|| popup_mismatch_finding(trigger, &kind, expected))
        })
        .collect()
}

/// The controlled element's popup role, or that of the first popup inside
/// it: triggers often control a wrapper around the popup.
fn popup_role(controlled: ElementRef) -> Option<&'static str> {
    std::iter::once(controlled)
        .chain(controlled.descendants())
        .filter_map(exposed_role)
        .map(|role| role.name)
        .find(|name| POPUP_ROLES.contains(name))
}

fn popup_mismatch_finding(trigger: ElementRef, kind: &str, expected: &str) -> Finding {
    let written = trigger.attr("aria-haspopup").unwrap_or_default().trim();
    Finding::error(
        "ARIA-WIDGET002",
        format!("aria-haspopup=\"{written}\" announces a {kind}, but the popup is a {expected}"),
    )
    .at(trigger)
    .help(format!(
        "set aria-haspopup=\"{expected}\" to match the popup's role"
    ))
}

/// ARIA-WIDGET003: suggestions in a list must be reachable from the field
/// that offers them.
pub fn check_autocomplete_popup(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_hidden(*el))
        .filter(|el| {
            matches!(
                token(*el, "aria-autocomplete").as_deref(),
                Some("list" | "both")
            )
        })
        .flat_map(|el| {
            [
                missing_suggestions_reference(el),
                missing_popup_announcement(el),
            ]
            .into_iter()
            .flatten()
            .map(move |(message, help)| Finding::error("ARIA-WIDGET003", message).at(el).help(help))
        })
        .collect()
}

fn missing_suggestions_reference(el: ElementRef) -> Problem {
    (!has_value(el, "aria-controls")).then(|| {
        (
            "list autocomplete has no aria-controls".to_string(),
            "reference the suggestions popup with aria-controls=\"<id of the listbox>\""
                .to_string(),
        )
    })
}

/// Only a combobox implies a listbox popup.
fn missing_popup_announcement(el: ElementRef) -> Problem {
    let is_text_field = has_exposed_role(el, TEXT_FIELD_ROLES);
    (is_text_field && popup_kind(el).is_none()).then(|| {
        (
            "list autocomplete on a text field has no aria-haspopup".to_string(),
            "set aria-haspopup to the suggestions popup's role, e.g. aria-haspopup=\"listbox\""
                .to_string(),
        )
    })
}

/// ARIA-WIDGET006: a popup trigger must be reachable from the keyboard.
/// Items of composite widgets are left out: their widget may manage focus
/// with `aria-activedescendant` or set roving `tabindex` from script.
pub fn check_unfocusable_popup_trigger(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.all()
        .filter(|el| popup_kind(*el).is_some() && !is_hidden(*el) && !is_aria_disabled(*el))
        .filter(|el| !is_focusable(*el) && !el.descendants().any(is_focusable))
        .filter(|el| exposed_role(*el).is_none_or(|role| role.required_context.is_empty()))
        .filter(|el| {
            !el.ancestors()
                .any(|ancestor| ancestor.has_attr("aria-activedescendant"))
        })
        .map(|el| {
            Finding::error(
                "ARIA-WIDGET006",
                format!(
                    "<{}> opens a popup but can't receive keyboard focus",
                    el.tag()
                ),
            )
            .at(el)
            .help(
                "use a <button> as the trigger, or add tabindex=\"0\" and open the popup on \
                 Enter and Space too",
            )
        })
        .collect()
}

/// ARIA-WIDGET007: readers move through a feed article by article, which
/// needs each article to be focusable.
pub fn check_unfocusable_feed_article(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    let tree = Tree::new(page);
    page.all()
        .filter(|el| has_exposed_role(*el, &["feed"]) && !is_hidden(*el))
        .flat_map(|feed| tree.owned(feed))
        .filter(|article| has_exposed_role(*article, &["article"]) && !is_focusable(*article))
        .map(|article| {
            Finding::error(
                "ARIA-WIDGET007",
                format!("<{}> in a feed isn't focusable", article.tag()),
            )
            .at(article)
            .help("add tabindex=\"0\" to each article of the feed so focus can move between them")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(check: fn(&RenderedPage, &CheckOptions) -> Vec<Finding>, html: &str) -> Vec<Finding> {
        check(&page_from_html(html), &CheckOptions::default())
    }

    const ARIA_12_COMBOBOX: &str = r#"<label for="tag_combo">Tag</label>
        <input type="text" id="tag_combo" role="combobox" aria-autocomplete="list"
          aria-haspopup="listbox" aria-expanded="true"
          aria-controls="popup_listbox" aria-activedescendant="selected_option">
        <ul role="listbox" id="popup_listbox">
          <li role="option" aria-selected="false">Zebra</li>
          <li role="option" id="selected_option" aria-selected="true">Zoom</li></ul>"#;

    #[test]
    fn aria_11_combobox_is_flagged() {
        let html = r#"<div role="combobox" aria-expanded="false" aria-owns="country-list">
              <input type="text" aria-label="Country"></div>
            <ul role="listbox" id="country-list"><li role="option">France</li></ul>"#;
        let findings = run(check_combobox_pattern, html);
        assert_eq!(findings.len(), 2, "{findings:?}");
    }

    #[rstest]
    #[case(
        r#"<input role="combobox" aria-controls="p"><ul role="listbox" id="p"></ul>"#,
        "no aria-expanded"
    )]
    #[case(r#"<input role="combobox" aria-expanded="false" aria-controls="p"><ul role="list" id="p"></ul>"#, "popup has role \"list\"")]
    #[case(r#"<input role="combobox" aria-expanded="false" aria-controls="p"><div role="dialog" id="p"></div>"#, "no aria-haspopup")]
    fn broken_comboboxes_are_flagged(#[case] html: &str, #[case] message: &str) {
        let findings = run(check_combobox_pattern, html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].message.contains(message), "{findings:?}");
    }

    #[rstest]
    #[case(ARIA_12_COMBOBOX)]
    #[case(r#"<div role="combobox" tabindex="0" aria-expanded="false" aria-controls="p">Pick</div><div role="listbox" id="p"></div>"#)]
    #[case(r#"<select><option>A</option></select>"#)]
    fn aria_12_comboboxes_are_not_flagged(#[case] html: &str) {
        assert!(run(check_combobox_pattern, html).is_empty(), "{html}");
    }

    #[rstest]
    #[case(
        r#"<button aria-haspopup="true" aria-controls="share-dialog">Share</button>
              <div role="dialog" id="share-dialog" aria-label="Share">...</div>"#
    )]
    #[case(
        r#"<button aria-haspopup="listbox" aria-controls="w">Pick</button>
              <div id="w"><ul role="menu"><li role="menuitem">A</li></ul></div>"#
    )]
    fn mismatched_popups_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_popup_role_match, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(
        r#"<button aria-haspopup="dialog" aria-controls="share-dialog">Share</button>
              <div role="dialog" id="share-dialog" aria-label="Share">...</div>"#
    )]
    #[case(r#"<button aria-haspopup="true" aria-controls="m">Menu</button><ul role="menu" id="m"></ul>"#)]
    #[case(r#"<button aria-haspopup="dialog" aria-controls="w">Open</button><div id="w">Loading</div>"#)]
    #[case(r#"<button aria-haspopup="false" aria-controls="m">Menu</button><div role="dialog" id="m"></div>"#)]
    #[case(r#"<button aria-haspopup="dialog" aria-controls="d">Open</button><div role="alertdialog" id="d"></div>"#)]
    fn matching_popups_are_not_flagged(#[case] html: &str) {
        assert!(run(check_popup_role_match, html).is_empty(), "{html}");
    }

    #[test]
    fn unwired_autocomplete_is_flagged() {
        let html = r#"<input type="search" aria-label="Search products" aria-autocomplete="list">
            <ul role="listbox" id="suggestions"></ul>"#;
        assert_eq!(run(check_autocomplete_popup, html).len(), 2);
    }

    #[rstest]
    #[case(r#"<input type="search" aria-label="Search products" aria-autocomplete="list"
              aria-controls="suggestions" aria-haspopup="listbox"><ul role="listbox" id="suggestions"></ul>"#)]
    #[case(ARIA_12_COMBOBOX)]
    #[case(r#"<input type="text" aria-autocomplete="inline">"#)]
    fn wired_autocomplete_is_not_flagged(#[case] html: &str) {
        assert!(run(check_autocomplete_popup, html).is_empty(), "{html}");
    }

    #[test]
    fn unfocusable_popup_trigger_is_flagged() {
        let html = r#"<span aria-haspopup="menu" onclick="openMenu()">Account</span>"#;
        assert_eq!(run(check_unfocusable_popup_trigger, html).len(), 1);
    }

    #[rstest]
    #[case(
        r#"<button aria-haspopup="menu" aria-expanded="false" aria-controls="m">Account</button>"#
    )]
    #[case(r#"<li aria-haspopup="true"><a href="/products">Products</a></li>"#)]
    #[case(r#"<ul role="menu"><li role="menuitem" aria-haspopup="menu">More</li></ul>"#)]
    #[case(r#"<span aria-haspopup="menu" aria-disabled="true">Account</span>"#)]
    #[case(r#"<span aria-haspopup="false">Account</span>"#)]
    fn focusable_popup_triggers_are_not_flagged(#[case] html: &str) {
        assert!(
            run(check_unfocusable_popup_trigger, html).is_empty(),
            "{html}"
        );
    }

    #[test]
    fn unfocusable_feed_article_is_flagged() {
        let html = r#"<div role="feed" aria-label="News">
            <article aria-labelledby="a1-title"><h2 id="a1-title">Title</h2></article></div>"#;
        assert_eq!(run(check_unfocusable_feed_article, html).len(), 1);
    }

    #[test]
    fn focusable_feed_article_is_not_flagged() {
        let html = r#"<div role="feed" aria-label="News" aria-busy="false">
            <article tabindex="0" aria-labelledby="a1-title"><h2 id="a1-title">Title</h2></article></div>
            <article>Not in a feed</article>"#;
        assert!(run(check_unfocusable_feed_article, html).is_empty());
    }
}
