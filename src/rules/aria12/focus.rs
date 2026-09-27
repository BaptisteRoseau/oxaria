//! Keyboard focus the markup can show: custom widgets that can't be
//! focused (ARIA-FOCUS001) and dialogs with nothing to focus
//! (ARIA-FOCUS006). Focus movement itself needs script.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{Role, first_valid_role, is_focusable, is_hidden};
use crate::rules::{CheckOptions, Finding};

use super::support::is_aria_disabled;
use super::tree::{exposed_role, has_exposed_role, has_presentational_children_ancestor};

/// Widget roles that take focus themselves. Items of composite widgets
/// (`option`, `menuitem`, `treeitem`, `gridcell`, ...) are left out: their
/// widget may manage focus with `aria-activedescendant` from elsewhere,
/// e.g. a combobox's listbox.
const FOCUSABLE_WIDGET_ROLES: &[(&str, &str)] = &[
    ("button", "<button>"),
    ("checkbox", "<input type=\"checkbox\">"),
    ("link", "<a href>"),
    ("radio", "<input type=\"radio\">"),
    ("searchbox", "<input type=\"search\">"),
    ("slider", "<input type=\"range\">"),
    ("spinbutton", "<input type=\"number\">"),
    ("switch", "<input type=\"checkbox\">"),
    ("tab", "<button>"),
    ("textbox", "<input> or <textarea>"),
];

const DIALOG_ROLES: &[&str] = &["dialog", "alertdialog"];

/// ARIA-FOCUS001: a custom widget must be focusable to be operable from
/// the keyboard.
pub fn check_unfocusable_widget(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !is_hidden(*el) && !is_aria_disabled(*el))
        .filter_map(|el| Some((el, explicit_widget(el)?)))
        .filter(|(el, _)| !is_focusable(*el) && !el.descendants().any(is_focusable))
        .filter(|(el, _)| !has_managed_focus(*el) && !has_presentational_children_ancestor(*el))
        .map(|(el, (role, native))| {
            Finding::error(
                "ARIA-FOCUS001",
                format!(
                    "<{}> with role=\"{}\" can't receive keyboard focus",
                    el.tag(),
                    role.name
                ),
            )
            .at(el)
            .help(format!(
                "use a native {native}, or add tabindex=\"0\" (tabindex=\"-1\" for all but the \
                 active item of a composite widget with arrow-key navigation)"
            ))
        })
        .collect()
}

fn explicit_widget(el: ElementRef) -> Option<(&'static Role, &'static str)> {
    let role = first_valid_role(el)?;
    let native = FOCUSABLE_WIDGET_ROLES
        .iter()
        .find(|(name, _)| *name == role.name)
        .map(|(_, native)| *native)?;
    exposed_role(el)
        .is_some_and(|exposed| exposed.name == role.name)
        .then_some((role, native))
}

fn has_managed_focus(el: ElementRef) -> bool {
    el.ancestors()
        .any(|ancestor| ancestor.has_attr("aria-activedescendant"))
}

/// ARIA-FOCUS006: a dialog needs something to put focus on. Empty dialogs
/// pass: script may fill them when they open.
pub fn check_dialog_without_focusable(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.all()
        .filter(|el| has_exposed_role(*el, DIALOG_ROLES) && !is_hidden(*el))
        .filter(|dialog| !dialog.text().is_empty())
        .filter(|dialog| !is_focusable(*dialog) && !dialog.descendants().any(is_focusable))
        .map(|dialog| {
            Finding::error(
                "ARIA-FOCUS006",
                format!("<{}> dialog has no focusable element", dialog.tag()),
            )
            .at(dialog)
            .help(
                "give the dialog a focusable control (e.g. a Close <button>) to move focus to \
                 when it opens, or tabindex=\"-1\" on its first static element",
            )
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

    #[rstest]
    #[case(r#"<div role="tablist" aria-label="Settings"><div role="tab" aria-selected="true">General</div>
              <div role="tab" aria-selected="false">Privacy</div></div>"#, 2)]
    #[case(r#"<span role="button" onclick="save()">Save</span>"#, 1)]
    #[case(r#"<a role="link">Home</a>"#, 1)]
    fn unfocusable_widgets_are_flagged(#[case] html: &str, #[case] count: usize) {
        assert_eq!(run(check_unfocusable_widget, html).len(), count, "{html}");
    }

    #[rstest]
    #[case(r#"<div role="tablist" aria-label="Settings"><div role="tab" tabindex="0" aria-selected="true">General</div>
              <div role="tab" tabindex="-1" aria-selected="false">Privacy</div></div>"#)]
    #[case(r#"<div role="radiogroup" tabindex="0" aria-activedescendant="r1"><div role="radio" id="r1">A</div></div>"#)]
    #[case(r#"<a role="link" aria-disabled="true">Export</a>"#)]
    #[case(r#"<li role="tab"><a href="/a">A</a></li>"#)]
    #[case(r#"<div role="textbox" contenteditable>x</div>"#)]
    #[case(r#"<div role="option">Items aren't checked</div>"#)]
    #[case(r#"<button role="tab">Native</button>"#)]
    #[case(r#"<button>Open <span role="button">x</span></button>"#)]
    fn focusable_widgets_are_not_flagged(#[case] html: &str) {
        assert!(run(check_unfocusable_widget, html).is_empty(), "{html}");
    }

    #[test]
    fn dialog_without_focusable_is_flagged() {
        let html = r#"<button onclick="closeDialog()">Close</button>
            <div role="dialog" aria-modal="true" aria-labelledby="dlg-title">
              <h2 id="dlg-title">Photo details</h2><p>Taken on 12 May 2026.</p></div>"#;
        assert_eq!(run(check_dialog_without_focusable, html).len(), 1);
    }

    #[rstest]
    #[case(
        r#"<div role="dialog" aria-modal="true" aria-labelledby="dlg-title">
              <h2 id="dlg-title">Photo details</h2><p>Taken on 12 May 2026.</p>
              <button onclick="closeDialog()">Close</button></div>"#
    )]
    #[case(r#"<div role="dialog" aria-label="Loading"></div>"#)]
    #[case(r#"<dialog><p>Closed</p></dialog>"#)]
    #[case(r#"<div role="dialog" tabindex="-1"><p>Static</p></div>"#)]
    fn dialogs_with_focusable_are_not_flagged(#[case] html: &str) {
        assert!(
            run(check_dialog_without_focusable, html).is_empty(),
            "{html}"
        );
    }
}
