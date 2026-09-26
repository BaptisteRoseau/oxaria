//! WCAG 2.1.1 / 2.4.7 / 4.1.2 checks for inline event handlers (`onclick="..."`).
//! Handlers attached from script (`addEventListener`) can't be seen without running it.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

const ACTIVATION_HANDLERS: &[&str] = &[
    "onclick",
    "ondblclick",
    "onmousedown",
    "onmouseup",
    "onpointerdown",
    "onpointerup",
    "ontouchstart",
    "ontouchend",
    "onkeydown",
    "onkeyup",
    "onkeypress",
];

/// `onclick` is left out: browsers fire it for Enter/Space on focusable elements too.
const POINTER_ONLY_HANDLERS: &[&str] = &[
    "ondblclick",
    "onmousedown",
    "onmouseup",
    "onpointerdown",
    "onpointerup",
    "ontouchstart",
    "ontouchend",
];

const KEYBOARD_HANDLERS: &[&str] = &[
    "onclick",
    "onkeydown",
    "onkeyup",
    "onkeypress",
    "onfocus",
    "onblur",
];

const NATIVE_CONTROLS: &[&str] = &[
    "a", "area", "button", "input", "select", "textarea", "summary",
];

/// F42: an element that navigates from a script handler instead of being a link is left out
/// of the links list and, unless focusable, the tab order.
pub fn check_emulated_link(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| !NATIVE_CONTROLS.contains(&el.tag()) && el.attr("role") != Some("link"))
        .filter(|el| navigates(*el))
        .map(|el| {
            Finding::error(
                "F42",
                format!(
                    "<{}> navigates from a script handler instead of being a link",
                    el.tag()
                ),
            )
            .at(el)
            .help(
                "use <a href=\"...\"> for navigation, which works with the keyboard \
                 and is listed as a link",
            )
        })
        .collect()
}

fn navigates(el: ElementRef) -> bool {
    handlers(el, ACTIVATION_HANDLERS)
        .any(|code| code.contains("location") || code.contains("window.open"))
}

/// F59: a `div` or `span` scripted into a control exposes no role, so assistive technology
/// doesn't announce it as one.
pub fn check_control_without_role(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| matches!(el.tag(), "div" | "span"))
        .filter(|el| el.attr("role").is_none_or(|role| role.trim().is_empty()))
        .filter(|el| handlers(*el, ACTIVATION_HANDLERS).next().is_some() && !navigates(*el))
        .map(|el| {
            Finding::error(
                "F59",
                format!("<{}> has an event handler but no role", el.tag()),
            )
            .at(el)
            .help(
                "use a native control such as <button>, or add the fitting role \
                 (e.g. role=\"checkbox\") plus tabindex=\"0\" and keyboard handling",
            )
        })
        .collect()
}

/// F54: a function bound only to pointer events (mousedown, touchstart, …) can't be invoked
/// from the keyboard.
pub fn check_pointer_only_handler(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| handlers(*el, KEYBOARD_HANDLERS).next().is_none())
        .filter_map(|el| Some((el, first_attr(el, POINTER_ONLY_HANDLERS)?)))
        .map(|(el, handler)| {
            Finding::error(
                "F54",
                format!("<{}> has {handler} but no keyboard equivalent", el.tag()),
            )
            .at(el)
            .help(
                "trigger the function from onclick on a native control, \
                 or add onkeydown/onfocus handlers doing the same",
            )
        })
        .collect()
}

/// F55: blurring an element as soon as it gets focus takes keyboard users' focus away.
pub fn check_focus_removed_on_focus(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| {
            el.attr("onfocus")
                .is_some_and(|code| code.contains("blur("))
        })
        .map(|el| {
            Finding::error(
                "F55",
                format!(
                    "<{}> removes its own focus as soon as it receives it",
                    el.tag()
                ),
            )
            .at(el)
            .help(
                "remove the blur() call; restyle the focus indicator in CSS \
                 if it looks out of place",
            )
        })
        .collect()
}

fn handlers<'a>(el: ElementRef<'a>, names: &'a [&str]) -> impl Iterator<Item = &'a str> {
    names.iter().filter_map(move |name| el.attr(name))
}

fn first_attr(el: ElementRef, names: &[&'static str]) -> Option<&'static str> {
    names.iter().copied().find(|name| el.has_attr(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(check: super::super::RuleCheck, html: &str) -> Vec<Finding> {
        check(&page_from_html(html), &CheckOptions::default())
    }

    #[test]
    fn scripted_span_link_is_flagged() {
        let findings = run(
            check_emulated_link,
            r#"<span onclick="location.href='newpage.html'">Fake link</span>"#,
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F42");
    }

    #[test]
    fn scripted_image_link_is_flagged() {
        let html =
            r#"<img src="go.gif" alt="go to the new page" onclick="location.href='newpage.html'">"#;
        assert_eq!(run(check_emulated_link, html).len(), 1);
    }

    #[test]
    fn real_links_and_role_link_are_not_flagged_as_emulated() {
        let html = r#"<a href="/a" onclick="location.href='/b'">A</a>
                      <span role="link" tabindex="0" onclick="location.href='/c'">C</span>"#;
        assert!(run(check_emulated_link, html).is_empty());
    }

    #[test]
    fn scripted_span_without_role_is_flagged() {
        let html = r#"<p><span onclick="toggleCheckbox('chkbox')">
                      <img src="unchecked.gif" id="chkbox" alt=""> Include Signature</span></p>"#;
        let findings = run(check_control_without_role, html);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F59");
    }

    #[test]
    fn scripted_div_with_role_is_not_flagged() {
        let html = r#"<div role="button" tabindex="0" onclick="go()">Go</div>"#;
        assert!(run(check_control_without_role, html).is_empty());
    }

    #[test]
    fn navigating_span_is_reported_as_emulated_link_only() {
        let html = r#"<span onclick="location.href='/x'">X</span>"#;
        assert!(run(check_control_without_role, html).is_empty());
    }

    #[test]
    fn mousedown_only_image_is_flagged() {
        let html =
            r#"<p><img onmousedown="nextPage();" src="nextarrow.gif" alt="Go to next page"></p>"#;
        let findings = run(check_pointer_only_handler, html);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F54");
        assert!(findings[0].message.contains("onmousedown"));
    }

    #[test]
    fn pointer_handler_with_keyboard_equivalent_is_not_flagged() {
        let html =
            r#"<div onmousedown="a()" onkeydown="a()"></div><button onclick="b()">B</button>"#;
        assert!(run(check_pointer_only_handler, html).is_empty());
    }

    #[test]
    fn blur_on_focus_is_flagged() {
        let html = r#"<input type="submit" onFocus="this.blur();">
                      <a href="link.html" onfocus="if(this.blur)this.blur();">Link Phrase</a>"#;
        let findings = run(check_focus_removed_on_focus, html);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].rule_id, "F55");
    }

    #[test]
    fn other_focus_handlers_are_not_flagged() {
        let html = r#"<input onfocus="showHint()">"#;
        assert!(run(check_focus_removed_on_focus, html).is_empty());
    }
}
