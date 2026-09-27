//! ARIA in HTML's "Allowed descendants of ARIA roles".

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{effective_role, first_valid_role, input_type, is_not_rendered};
use crate::rules::{CheckOptions, Finding};

use super::markup::start_tag;

/// Roles allowing no interactive content and no `tabindex` descendants.
const CONTROL_ROLES: &[&str] = &[
    "button",
    "checkbox",
    "link",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "switch",
    "tab",
];

/// Roles whose "Kind of content" is interactive content.
const INTERACTIVE_ROLES: &[&str] = &[
    "button",
    "checkbox",
    "combobox",
    "grid",
    "gridcell",
    "link",
    "listbox",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "scrollbar",
    "searchbox",
    "slider",
    "spinbutton",
    "switch",
    "tab",
    "textbox",
    "treeitem",
];

/// Roles allowing no `main` element descendants.
const NO_MAIN_ROLES: &[&str] = &[
    "alert",
    "article",
    "banner",
    "blockquote",
    "caption",
    "cell",
    "columnheader",
    "combobox",
    "complementary",
    "contentinfo",
    "directory",
    "feed",
    "figure",
    "gridcell",
    "listitem",
    "log",
    "main",
    "marquee",
    "navigation",
    "note",
    "region",
    "rowheader",
    "search",
    "searchbox",
    "spinbutton",
    "status",
    "textbox",
    "timer",
    "toolbar",
];

/// A role and the element it can't contain.
const FORBIDDEN_ELEMENTS: &[(&str, &str)] = &[
    ("form", "form"),
    ("meter", "meter"),
    ("progressbar", "progress"),
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Violation {
    Interactive,
    Tabindex,
    Element,
}

/// HTMLARIA017: content a role's allowed descendants exclude, reported
/// once, against the nearest ancestor excluding it. The `main`, `form`,
/// `meter` and `progress` limits only apply to explicit roles: HTML's own
/// content model already covers the native elements.
pub fn check_disallowed_descendants(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_candidate(*el) && !is_not_rendered(*el))
        .filter_map(|el| {
            el.ancestors()
                .find_map(|container| violation(container, el).map(|kind| (container, kind)))
                .map(|(container, kind)| descendant_finding(container, el, kind))
        })
        .collect()
}

fn is_candidate(el: ElementRef) -> bool {
    is_interactive(el)
        || el.has_attr("tabindex")
        || matches!(el.tag(), "main" | "form" | "meter" | "progress")
}

fn violation(container: ElementRef, el: ElementRef) -> Option<Violation> {
    let role = effective_role(container)?.name;
    let explicit = first_valid_role(container).map(|role| role.name);
    match role {
        _ if CONTROL_ROLES.contains(&role) && is_interactive(el) => Some(Violation::Interactive),
        _ if CONTROL_ROLES.contains(&role) && el.has_attr("tabindex") => Some(Violation::Tabindex),
        "img" if is_interactive(el) => Some(Violation::Interactive),
        _ if explicit.is_some_and(|explicit| forbids_element(explicit, el.tag())) => {
            Some(Violation::Element)
        }
        _ => None,
    }
}

fn forbids_element(role: &str, tag: &str) -> bool {
    (tag == "main" && NO_MAIN_ROLES.contains(&role)) || FORBIDDEN_ELEMENTS.contains(&(role, tag))
}

/// HTML's interactive content, or an element with an interactive role.
fn is_interactive(el: ElementRef) -> bool {
    let is_interactive_content = match el.tag() {
        "a" => el.has_attr("href"),
        "audio" | "video" => el.has_attr("controls"),
        "img" => el.has_attr("usemap"),
        "input" => input_type(el) != "hidden",
        tag => matches!(
            tag,
            "button" | "details" | "embed" | "iframe" | "label" | "select" | "textarea"
        ),
    };
    is_interactive_content
        || first_valid_role(el).is_some_and(|role| INTERACTIVE_ROLES.contains(&role.name))
}

fn descendant_finding(container: ElementRef, el: ElementRef, kind: Violation) -> Finding {
    let role = effective_role(container).map_or("", |role| role.name);
    let descendant = start_tag(el, &["type", "role", "tabindex"]);
    let limit = match kind {
        Violation::Interactive => "interactive descendants".to_string(),
        Violation::Tabindex => "descendants with tabindex".to_string(),
        Violation::Element => format!("<{}> descendants", el.tag()),
    };
    Finding::error(
        "HTMLARIA017",
        format!(
            "{descendant} is inside {}, whose {role} role allows no {limit}",
            start_tag(container, &["role"])
        ),
    )
    .at(el)
    .help(descendant_help(container, el, kind, role))
}

fn descendant_help(container: ElementRef, el: ElementRef, kind: Violation, role: &str) -> String {
    let (tag, container_tag) = (el.tag(), container.tag());
    match kind {
        Violation::Interactive => format!(
            "move the <{tag}> out of the <{container_tag}> (e.g. next to it): nested controls \
             are announced as part of the {role} and can't be operated on their own"
        ),
        Violation::Tabindex => format!(
            "remove tabindex from the <{tag}>: the {role}'s content is announced as part of it \
             and shouldn't be focusable on its own"
        ),
        Violation::Element => {
            format!("move the <{tag}> out of the element with role=\"{role}\", or remove that role")
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(html: &str) -> Vec<Finding> {
        check_disallowed_descendants(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(r#"<button><div role="button">x</div></button>"#)]
    #[case(r#"<div role="button"><button>x</button></div>"#)]
    #[case(r#"<div role="link"><textarea>x</textarea></div>"#)]
    #[case(r#"<a href="/"><span tabindex="-1">x</span></a>"#)]
    #[case(r#"<div role="tab">T <button>Close</button></div>"#)]
    #[case(r#"<div role="img"><a href="/">x</a></div>"#)]
    #[case(r#"<div role="article"><main>x</main></div>"#)]
    #[case(r#"<div role="form"><form>x</form></div>"#)]
    #[case(r#"<div role="progressbar"><progress></progress></div>"#)]
    #[case(r#"<div role="meter"><meter></meter></div>"#)]
    #[case(r#"<a href="/"><input type="checkbox"></a>"#)]
    fn disallowed_descendants_are_flagged(#[case] html: &str) {
        let findings = run(html);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "HTMLARIA017");
    }

    #[rstest]
    #[case(r#"<div role="button" tabindex="0">x</div>"#)]
    #[case("<button>x</button>")]
    #[case(r#"<div role="link" tabindex="0">x</div><textarea>x</textarea>"#)]
    #[case(r#"<a href="/"><svg role="img"></svg><span>x</span></a>"#)]
    #[case(r#"<div role="img"><span tabindex="-1">x</span></div>"#)]
    #[case(r#"<button><input type="hidden"></button>"#)]
    #[case(r#"<button><a>no href</a></button>"#)]
    #[case(r#"<div role="button"><span hidden><button>x</button></span></div>"#)]
    #[case(r#"<article><main>x</main></article>"#)]
    #[case(r#"<div role="navigation"><form>x</form></div>"#)]
    #[case(r#"<nav><a href="/">x</a><button>y</button></nav>"#)]
    fn allowed_descendants_are_not_flagged(#[case] html: &str) {
        assert!(run(html).is_empty());
    }

    #[test]
    fn each_nested_control_is_reported_against_its_nearest_container() {
        let findings = run(r#"<a href="/"><button><span tabindex="0">x</span></button></a>"#);
        let messages: Vec<_> = findings.iter().map(|f| f.message.as_str()).collect();
        assert_eq!(
            messages,
            [
                "<button> is inside <a>, whose link role allows no interactive descendants",
                r#"<span tabindex="0"> is inside <button>, whose button role allows no descendants with tabindex"#,
            ]
        );
    }

    #[test]
    fn interactive_help_names_both_elements() {
        let findings = run(r#"<div role="button"><button>x</button></div>"#);
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with("move the <button> out of the <div>")
        );
    }
}
