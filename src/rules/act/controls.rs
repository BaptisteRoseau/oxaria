//! ACT rules requiring buttons, links, menu items, `summary` elements and
//! form fields to have a non-empty accessible name.

use super::name::accessible_name;
use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{effective_role, first_valid_role, input_type, is_hidden};
use crate::rules::{CheckOptions, Finding};

const FORM_FIELD_ROLES: &[&str] = &[
    "checkbox",
    "combobox",
    "listbox",
    "menuitemcheckbox",
    "menuitemradio",
    "radio",
    "searchbox",
    "slider",
    "spinbutton",
    "switch",
    "textbox",
];

/// `input` types without a role in ARIA in HTML.
const ROLELESS_FIELD_TYPES: &[&str] = &[
    "color",
    "date",
    "datetime-local",
    "file",
    "month",
    "password",
    "time",
    "week",
];

/// 97a4e1: every element exposed as a button, except `input type="image"`
/// (see 59796f), needs a non-empty accessible name.
pub fn check_button_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    unnamed(page, |el| {
        has_role(el, "button") && !(el.tag() == "input" && input_type(el) == "image")
    })
    .map(|button| {
        Finding::error(
            "97a4e1",
            format!("{} has no accessible name", markup(button)),
        )
        .at(button)
        .help(button_help(button))
    })
    .collect()
}

fn button_help(button: ElementRef) -> &'static str {
    match button.tag() {
        "input" => "give it a value=\"...\" naming its action",
        "button" => {
            "give it visible text, or aria-label=\"...\" if it only shows an icon; \
             its value attribute doesn't name it"
        }
        _ => "give it text content, or aria-label=\"...\" if it only shows an icon",
    }
}

/// c487ae: every element exposed as a link needs a non-empty accessible name.
pub fn check_link_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    unnamed(page, |el| has_role(el, "link"))
        .map(|link| {
            Finding::error("c487ae", format!("{} has no accessible name", markup(link)))
                .at(link)
                .help(link_help(link))
        })
        .collect()
}

fn link_help(link: ElementRef) -> &'static str {
    match (link.tag(), has_image(link)) {
        ("area", _) => "add alt=\"...\" saying where the area links to",
        (_, true) => {
            "give the image an alt=\"...\" saying where the link goes \
             (alt=\"\" marks it decorative, leaving the link unnamed)"
        }
        _ => "give it text saying where it goes, or aria-label=\"...\" if it only shows an icon",
    }
}

fn has_image(el: ElementRef) -> bool {
    el.descendants()
        .any(|descendant| matches!(descendant.tag(), "img" | "svg"))
}

/// m6b1q3: every element exposed as a `menuitem` needs a non-empty
/// accessible name.
pub fn check_menuitem_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    unnamed(page, |el| has_role(el, "menuitem"))
        .map(|item| {
            Finding::error("m6b1q3", format!("{} has no accessible name", markup(item)))
                .at(item)
                .help(
                    "give it visible text, or aria-label=\"...\" if it only shows an icon \
                 (an icon with alt=\"\" gives it no name)",
                )
        })
        .collect()
}

/// 2t702h: the `summary` that toggles a `details` element needs a non-empty
/// accessible name besides its disclosure marker.
pub fn check_summary_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    unnamed(page, |el| is_details_summary(el) && has_no_explicit_role(el))
        .map(|summary| {
            Finding::error(
                "2t702h",
                format!("{} has no accessible name", markup(summary)),
            )
            .at(summary)
            .help("give the <summary> text saying what the <details> reveals, e.g. <summary>Opening times</summary>")
        })
        .collect()
}

/// The first `summary` child of a `details`: the one that toggles it.
fn is_details_summary(el: ElementRef) -> bool {
    el.tag() == "summary"
        && el.parent().is_some_and(|parent| {
            parent.tag() == "details"
                && parent
                    .children()
                    .find(|child| child.tag() == "summary")
                    .is_some_and(|first| first == el)
        })
}

/// A presentational role doesn't count: the summary is focusable, so
/// WAI-ARIA's conflict resolution ignores it.
fn has_no_explicit_role(el: ElementRef) -> bool {
    first_valid_role(el).is_none_or(|role| role.is_presentational())
}

/// e086e5: every form field (an element with a form field role, or an
/// `input` type without a role) needs a non-empty accessible name.
pub fn check_form_field_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    unnamed(page, is_form_field)
        .map(|field| {
            Finding::error(
                "e086e5",
                format!("{} has no accessible name", markup(field)),
            )
            .at(field)
            .help(form_field_help(field))
        })
        .collect()
}

fn is_form_field(el: ElementRef) -> bool {
    match effective_role(el) {
        Some(role) => FORM_FIELD_ROLES.contains(&role.name),
        None => el.tag() == "input" && ROLELESS_FIELD_TYPES.contains(&input_type(el)),
    }
}

fn form_field_help(field: ElementRef) -> String {
    let label = match field.attr("id") {
        Some(id) => format!("<label for=\"{id}\">...</label>"),
        None => "a wrapping <label>...</label>".to_string(),
    };
    match field.tag() {
        "input" | "select" | "textarea" => {
            format!("give it a visible label with {label}, or aria-labelledby=\"...\"")
        }
        _ => "point aria-labelledby=\"...\" at its visible label, or add aria-label=\"...\"; \
              a <label> only names native form fields"
            .to_string(),
    }
}

fn unnamed<'a>(
    page: &'a RenderedPage,
    applies: impl Fn(ElementRef) -> bool + 'a,
) -> impl Iterator<Item = ElementRef<'a>> {
    page.all()
        .filter(move |el| applies(*el) && !is_hidden(*el))
        .filter(|el| accessible_name(*el).is_empty())
}

fn has_role(el: ElementRef, name: &str) -> bool {
    effective_role(el).is_some_and(|role| role.name == name)
}

fn markup(el: ElementRef) -> String {
    let attribute = ["type", "role", "href"]
        .into_iter()
        .find_map(|name| Some((name, el.attr(name)?)));
    match attribute {
        Some((name, value)) => format!("<{} {name}=\"{value}\">", el.tag()),
        None => format!("<{}>", el.tag()),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;
    use crate::rules::RuleCheck;

    fn count(check: RuleCheck, html: &str) -> usize {
        check(&page_from_html(html), &CheckOptions::default()).len()
    }

    #[rstest]
    #[case("<button></button>", 1)]
    #[case(r#"<button type="button" value="read more"></button>"#, 1)]
    #[case(r#"<span role="button"></span>"#, 1)]
    #[case(r#"<button role="none"></button>"#, 1)]
    #[case(r#"<input type="button">"#, 1)]
    #[case(r#"<button><svg aria-hidden="true"></svg></button>"#, 1)]
    #[case("<button>My button</button>", 0)]
    #[case(r#"<input type="submit" value="Submit">"#, 0)]
    #[case(r#"<input type="submit">"#, 0)]
    #[case(r#"<input type="reset">"#, 0)]
    #[case(r#"<button aria-label="My button"></button>"#, 0)]
    #[case(r#"<span role="button" aria-label="My button"></span>"#, 0)]
    #[case("<button disabled>Delete</button>", 0)]
    #[case(r#"<input type="image" value="download">"#, 0)]
    #[case(r#"<button hidden></button>"#, 0)]
    #[case(r#"<button role="link">take me somewhere</button>"#, 0)]
    #[case(r#"<button role="none" disabled></button>"#, 0)]
    #[case(
        r#"<button><span class="sr-only">Close</span><svg aria-hidden="true"></svg></button>"#,
        0
    )]
    fn buttons_97a4e1(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_button_name, html), expected, "{html}");
    }

    #[rstest]
    #[case(r#"<a href="http://www.w3.org/WAI"></a>"#, 1)]
    #[case(r#"<a href="/"><img src="w3c.png" alt=""></a>"#, 1)]
    #[case(r#"<a href="/"><img src="w3c.png" role="presentation"></a>"#, 1)]
    #[case(r#"<a href="/"><img src="w3c.png" title=""></a>"#, 1)]
    #[case(
        r#"<a href="/"><img src="w3c.png" aria-labelledby="id1"></a><div id="id1"></div>"#,
        1
    )]
    #[case(r#"<map name="m"><area shape="rect" href="sun.htm"></map>"#, 1)]
    #[case(r#"<a href="/" role="none"> </a>"#, 1)]
    #[case(
        r#"<a href="/" role="doc-biblioref"><img src="act.png" alt=""></a>"#,
        1
    )]
    #[case(r#"<a href="/"> Web Accessibility Initiative (WAI) </a>"#, 0)]
    #[case(r#"<div role="link" tabindex="0"> WAI </div>"#, 0)]
    #[case(r#"<a href="/"><img src="w3c.png" aria-label="WAI"></a>"#, 0)]
    #[case(r#"<a href="/" title="WAI"><img src="w3c.png" alt=""></a>"#, 0)]
    #[case(r#"<a href="/"><img src="w3c.png" title="WAI"></a>"#, 0)]
    #[case(
        r#"<a href="/"><img src="w3c.png" aria-labelledby="i"></a><div id="i">WAI</div>"#,
        0
    )]
    #[case(
        r#"<map name="m"><area shape="rect" href="sun.htm" alt="Sun"></map>"#,
        0
    )]
    #[case(r#"<a href="/" role="button">WAI</a>"#, 0)]
    #[case(r#"<a aria-hidden="true" href="/">WAI</a>"#, 0)]
    #[case("<a></a>", 0)]
    fn links_c487ae(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_link_name, html), expected, "{html}");
    }

    #[rstest]
    #[case(
        r#"<div role="menu"><button role="menuitem"><img src="f.svg" alt=""></button></div>"#,
        1
    )]
    #[case(
        r#"<div role="menu"><button role="menuitem">New file</button></div>"#,
        0
    )]
    #[case(r#"<div role="menu"><button role="menuitem" aria-label="New"><img src="f.svg" alt=""></button></div>"#, 0)]
    #[case(r#"<div role="menu"><button role="menuitem" aria-labelledby="n"><span id="n" hidden>New</span></button></div>"#, 0)]
    #[case(r#"<div role="menu"><button role="menuitem" title="New"><img src="f.svg" alt=""></button></div>"#, 0)]
    #[case(r##"<menu><li><a href="#">New file</a></li></menu>"##, 0)]
    #[case(
        r#"<div role="menu" hidden><button role="menuitem"></button></div>"#,
        0
    )]
    fn menuitems_m6b1q3(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_menuitem_name, html), expected, "{html}");
    }

    #[rstest]
    #[case("<details><summary></summary><p>We are open 24/7.</p></details>", 1)]
    #[case(r#"<details><summary role="none"></summary><p>Open</p></details>"#, 1)]
    #[case(
        "<details><summary></summary><summary>Opening times</summary></details>",
        1
    )]
    #[case("<details><summary>Opening times</summary><p>Open</p></details>", 0)]
    #[case(
        r#"<details><summary aria-label="Opening times"></summary></details>"#,
        0
    )]
    #[case(r#"<span id="o">Opening times</span><details><summary aria-labelledby="o"></summary></details>"#, 0)]
    #[case("<details><p>Open</p><summary>Opening times</summary></details>", 0)]
    #[case(
        "<details><summary>Opening times</summary><summary></summary></details>",
        0
    )]
    #[case("<summary></summary>", 0)]
    #[case("<details><div><summary></summary></div></details>", 0)]
    #[case(r#"<details><summary role="button"></summary></details>"#, 0)]
    #[case(r#"<details hidden><summary></summary></details>"#, 0)]
    fn summaries_2t702h(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_summary_name, html), expected, "{html}");
    }

    #[rstest]
    #[case("<div>last name</div><input>", 1)]
    #[case("<input disabled>", 1)]
    #[case(r#"<input aria-label=" ">"#, 1)]
    #[case(
        r#"<div id="c"></div><select aria-labelledby="c"><option>England</option></select>"#,
        1
    )]
    #[case(r#"<label>first name <div role="textbox"></div></label>"#, 1)]
    #[case(
        r#"<label for="f">first name</label><div role="textbox" id="f"></div>"#,
        1
    )]
    #[case(r#"<div role="textbox">first name</div>"#, 1)]
    #[case(
        r#"<input type="checkbox" role="menuitemcheckbox"><span aria-hidden="true">Ketchup</span>"#,
        1
    )]
    #[case(r#"<label>Date of birth</label><input type="date">"#, 1)]
    #[case(r#"<select><option>England</option></select>"#, 1)]
    #[case("<label>first name <input></label>", 0)]
    #[case(r#"<input aria-label="last name" disabled>"#, 0)]
    #[case(
        r#"<label for="c">Country</label><select id="c"><option>England</option></select>"#,
        0
    )]
    #[case(
        r#"<div id="c">Country</div><textarea aria-labelledby="c"></textarea>"#,
        0
    )]
    #[case(r#"<input placeholder="Your search query">"#, 0)]
    #[case(
        r#"<div aria-label="country" role="combobox" aria-disabled="true">England</div>"#,
        0
    )]
    #[case(r#"<div role="checkbox">I agree to the terms.</div>"#, 0)]
    #[case(r#"<input type="checkbox" role="menuitemcheckbox" aria-labelledby="k"><span id="k" aria-hidden="true">Ketchup</span>"#, 0)]
    #[case(r#"<label>Favorite color <input type="color"></label>"#, 0)]
    #[case(r#"<input aria-label="first" style="display:none" hidden>"#, 0)]
    #[case(r#"<input disabled aria-hidden="true">"#, 0)]
    #[case(r#"<select role="none" disabled><option>Volvo</option></select>"#, 0)]
    #[case(r#"<input type="hidden">"#, 0)]
    #[case(r#"<input type="submit">"#, 0)]
    fn form_fields_e086e5(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_form_field_name, html), expected, "{html}");
    }

    #[test]
    fn messages_quote_the_element() {
        let page = page_from_html(r#"<input type="date">"#);
        let findings = check_form_field_name(&page, &CheckOptions::default());
        assert_eq!(
            findings[0].message,
            r#"<input type="date"> has no accessible name"#
        );
    }
}
