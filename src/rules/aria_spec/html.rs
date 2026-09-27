//! ARIA in HTML (W3C Recommendation of 11 August 2026, w3c/html-aria commit
//! `e277aa3`): each element's implicit role, the `aria-*` attributes
//! authors may set on it, and the native attributes with an
//! `aria-*` equivalent. Conditional rows ("`a` with `href`", "`img` with
//! `alt=""`", ...) are resolved against the element.

use crate::page::ElementRef;

use super::element::{effective_role, first_valid_role};
use super::roles::{Role, role};

/// "No `role` or `aria-*` attributes" in the document conformance table
/// (plus `input type=hidden`, which [`allowed_aria`] handles).
const NO_ROLE_OR_ARIA_ELEMENTS: &[&str] = &[
    "base", "col", "colgroup", "head", "link", "map", "meta", "noscript", "param", "script",
    "slot", "source", "style", "template", "title", "track",
];

/// Roles ARIA in HTML asks conformance checkers to warn about: WAI-ARIA's
/// deprecated `directory` and DPub-ARIA's deprecated ones.
pub const DEPRECATED_ROLES: &[&str] = &["directory", "doc-biblioentry", "doc-endnote"];

/// A native attribute with an `aria-*` equivalent, on the elements that
/// allow it (`None`: any).
#[derive(Debug)]
pub struct NativeEquivalent {
    pub native: &'static str,
    pub elements: Option<&'static [&'static str]>,
}

pub const NATIVE_EQUIVALENTS: &[NativeEquivalent] = &[
    NativeEquivalent {
        native: "checked",
        elements: Some(&["input"]),
    },
    NativeEquivalent {
        native: "disabled",
        elements: Some(&[
            "button", "fieldset", "input", "optgroup", "option", "select", "textarea",
        ]),
    },
    NativeEquivalent {
        native: "hidden",
        elements: None,
    },
    NativeEquivalent {
        native: "placeholder",
        elements: Some(&["input", "textarea"]),
    },
    NativeEquivalent {
        native: "max",
        elements: Some(&["input", "meter", "progress"]),
    },
    NativeEquivalent {
        native: "min",
        elements: Some(&["input", "meter"]),
    },
    NativeEquivalent {
        native: "readonly",
        elements: Some(&["input", "textarea"]),
    },
    NativeEquivalent {
        native: "contenteditable",
        elements: None,
    },
    NativeEquivalent {
        native: "required",
        elements: Some(&["input", "select", "textarea"]),
    },
    NativeEquivalent {
        native: "colspan",
        elements: Some(&["td", "th"]),
    },
    NativeEquivalent {
        native: "rowspan",
        elements: Some(&["td", "th"]),
    },
];

/// The `aria-*` attributes an author may set on an element. Prohibited
/// attributes of the role stay prohibited in every case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedAria {
    NoAria,
    AriaHiddenOnly,
    /// `aria-hidden="true"` and nothing else.
    AriaHiddenTrueOnly,
    GlobalOnly,
    GlobalAnd(&'static [&'static str]),
    /// Globals and those of this role, whatever the element's own role.
    GlobalAndRole(&'static str),
    /// Globals and those of the element's role: explicit, else implicit.
    GlobalAndEffectiveRole,
}

const INPUT_TYPES: &[&str] = &[
    "button",
    "checkbox",
    "color",
    "date",
    "datetime-local",
    "email",
    "file",
    "hidden",
    "image",
    "month",
    "number",
    "password",
    "radio",
    "range",
    "reset",
    "search",
    "submit",
    "tel",
    "text",
    "time",
    "url",
    "week",
];

/// `None` for "No corresponding role", for `summary` (which varies by user
/// agent) and for `svg`, whose `graphics-document` isn't a WAI-ARIA 1.2 role.
pub fn implicit_role(el: ElementRef) -> Option<&'static Role> {
    role(implicit_role_name(el)?)
}

/// The `type` of an `input`, lowercased; a missing or unknown one is `text`.
pub fn input_type(el: ElementRef) -> &'static str {
    el.attr("type")
        .and_then(|value| {
            INPUT_TYPES
                .iter()
                .find(|t| t.eq_ignore_ascii_case(value.trim()))
        })
        .copied()
        .unwrap_or("text")
}

pub fn allowed_aria(el: ElementRef) -> AllowedAria {
    use AllowedAria::{
        AriaHiddenOnly, AriaHiddenTrueOnly, GlobalAnd, GlobalAndEffectiveRole, GlobalAndRole,
        GlobalOnly, NoAria,
    };
    match el.tag() {
        "datalist" | "html" => NoAria,
        "input" if input_type(el) == "hidden" => NoAria,
        "button" if is_first_child_of_select(el) => NoAria,
        "selectedcontent" if has_ancestor(el, "select") => NoAria,
        "br" | "wbr" | "picture" => AriaHiddenOnly,
        "img" if is_decorative_img(el) => AriaHiddenTrueOnly,
        "caption" | "label" | "legend" => GlobalOnly,
        "body" => GlobalAndRole("generic"),
        "dd" => GlobalAndRole("definition"),
        "audio" | "video" => GlobalAndRole("application"),
        "summary" if is_details_summary(el) => GlobalAnd(&["aria-disabled", "aria-haspopup"]),
        "input" => input_allowed_aria(el),
        tag if NO_ROLE_OR_ARIA_ELEMENTS.contains(&tag) => NoAria,
        _ => GlobalAndEffectiveRole,
    }
}

/// A non-empty `aria-labelledby` target, `aria-label` or `title`: the
/// naming methods that make an `img` or a `section` named.
pub fn has_author_name(el: ElementRef) -> bool {
    let labelledby_text = el
        .attr("aria-labelledby")
        .map(|ids| el.page().ids_text(ids));
    [
        labelledby_text.as_deref(),
        el.attr("aria-label"),
        el.attr("title"),
    ]
    .into_iter()
    .flatten()
    .any(|name| !name.trim().is_empty())
}

fn implicit_role_name(el: ElementRef) -> Option<&'static str> {
    match el.tag() {
        "a" | "area" if el.has_attr("href") => Some("link"),
        "a" | "area" | "b" | "bdi" | "bdo" | "body" | "data" | "div" | "html" | "i" | "pre"
        | "q" | "samp" | "selectedcontent" | "small" | "span" | "u" => Some("generic"),
        "address" | "details" | "fieldset" | "hgroup" | "optgroup" => Some("group"),
        "article" => Some("article"),
        "aside" => Some("complementary"),
        "blockquote" => Some("blockquote"),
        "button" => Some("button"),
        "caption" => Some("caption"),
        "code" => Some("code"),
        "datalist" => Some("listbox"),
        "del" | "s" => Some("deletion"),
        "dfn" => Some("term"),
        "dialog" => Some("dialog"),
        "em" => Some("emphasis"),
        "figure" => Some("figure"),
        "footer" => Some(scoped_or(el, "contentinfo")),
        "form" => Some("form"),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some("heading"),
        "header" => Some(scoped_or(el, "banner")),
        "hr" => Some("separator"),
        "img" if is_decorative_img(el) => Some("none"),
        "img" => Some("img"),
        "input" => input_implicit_role(el),
        "ins" => Some("insertion"),
        "li" if parent_is_list(el) => Some("listitem"),
        "li" => Some("generic"),
        "main" => Some("main"),
        "math" => Some("math"),
        "menu" | "ol" | "ul" => Some("list"),
        "meter" => Some("meter"),
        "nav" => Some("navigation"),
        "option" if has_ancestor(el, "select") || has_ancestor(el, "datalist") => Some("option"),
        "output" => Some("status"),
        "p" => Some("paragraph"),
        "progress" => Some("progressbar"),
        "search" => Some("search"),
        "section" if has_author_name(el) => Some("region"),
        "section" => Some("generic"),
        "select" if is_list_box_select(el) => Some("listbox"),
        "select" => Some("combobox"),
        "strong" => Some("strong"),
        "sub" => Some("subscript"),
        "sup" => Some("superscript"),
        "table" => Some("table"),
        "tbody" | "tfoot" | "thead" => Some("rowgroup"),
        "td" => table_role(el).map(data_cell_role),
        "th" => table_role(el).map(|_| header_cell_role(el)),
        "textarea" => Some("textbox"),
        "time" => Some("time"),
        "tr" => Some("row"),
        tag if is_custom_element(tag) => Some("generic"),
        _ => None,
    }
}

fn input_implicit_role(el: ElementRef) -> Option<&'static str> {
    let has_list = el.has_attr("list");
    match input_type(el) {
        "button" | "image" | "reset" | "submit" => Some("button"),
        "checkbox" => Some("checkbox"),
        "radio" => Some("radio"),
        "range" => Some("slider"),
        "number" => Some("spinbutton"),
        "email" | "search" | "tel" | "text" | "url" if has_list => Some("combobox"),
        "search" => Some("searchbox"),
        "email" | "tel" | "text" | "url" => Some("textbox"),
        _ => None,
    }
}

fn input_allowed_aria(el: ElementRef) -> AllowedAria {
    use AllowedAria::{GlobalAnd, GlobalAndEffectiveRole, GlobalAndRole};
    match input_type(el) {
        "color" => GlobalAnd(&["aria-disabled"]),
        "file" => GlobalAnd(&["aria-disabled", "aria-invalid", "aria-required"]),
        "date" | "datetime-local" | "month" | "password" | "time" | "week" => {
            GlobalAndRole("textbox")
        }
        _ => GlobalAndEffectiveRole,
    }
}

/// `alt=""` and no other naming method.
fn is_decorative_img(el: ElementRef) -> bool {
    el.tag() == "img" && el.attr("alt") == Some("") && !has_author_name(el)
}

/// `header`/`footer` inside sectioning content or a sectioning role lose
/// their landmark role.
fn is_scoped_to_sectioning_content(el: ElementRef) -> bool {
    el.ancestors().any(|ancestor| {
        matches!(
            ancestor.tag(),
            "article" | "aside" | "main" | "nav" | "section"
        ) || first_valid_role(ancestor).is_some_and(|role| {
            matches!(
                role.name,
                "article" | "complementary" | "main" | "navigation" | "region"
            )
        })
    })
}

fn scoped_or(el: ElementRef, landmark: &'static str) -> &'static str {
    match is_scoped_to_sectioning_content(el) {
        true => "generic",
        false => landmark,
    }
}

fn parent_is_list(el: ElementRef) -> bool {
    el.parent()
        .and_then(effective_role)
        .is_some_and(|role| role.is_a("list"))
}

/// The role of the nearest ancestor `table`, when it is exposed as
/// `table`, `grid` or `treegrid`.
fn table_role(el: ElementRef) -> Option<&'static str> {
    let table = el.ancestors().find(|ancestor| ancestor.tag() == "table")?;
    effective_role(table)
        .map(|role| role.name)
        .filter(|name| matches!(*name, "table" | "grid" | "treegrid"))
}

fn data_cell_role(table_role: &str) -> &'static str {
    match table_role {
        "table" => "cell",
        _ => "gridcell",
    }
}

/// HTML-AAM's header scope, simplified: an explicit `scope`, else a row
/// header when its row holds data cells.
fn header_cell_role(th: ElementRef) -> &'static str {
    let scope = th
        .attr("scope")
        .map(|scope| scope.trim().to_ascii_lowercase());
    match scope.as_deref() {
        Some("row" | "rowgroup") => "rowheader",
        Some("col" | "colgroup") => "columnheader",
        _ if row_has_data_cells(th) => "rowheader",
        _ => "columnheader",
    }
}

fn row_has_data_cells(cell: ElementRef) -> bool {
    cell.parent()
        .is_some_and(|row| row.children().any(|sibling| sibling.tag() == "td"))
}

fn is_list_box_select(el: ElementRef) -> bool {
    let size = el
        .attr("size")
        .and_then(|size| size.trim().parse::<u32>().ok());
    el.has_attr("multiple") || size.is_some_and(|size| size > 1)
}

fn is_first_child_of_select(el: ElementRef) -> bool {
    el.parent().is_some_and(|parent| {
        parent.tag() == "select"
            && element_children(parent)
                .next()
                .is_some_and(|first| first == el)
    })
}

/// The first `summary` child of a `details`.
pub(super) fn is_details_summary(el: ElementRef) -> bool {
    el.parent().is_some_and(|parent| {
        parent.tag() == "details"
            && element_children(parent)
                .find(|child| child.tag() == "summary")
                .is_some_and(|summary| summary == el)
    })
}

fn has_ancestor(el: ElementRef, tag: &str) -> bool {
    el.ancestors().any(|ancestor| ancestor.tag() == tag)
}

fn element_children<'a>(el: ElementRef<'a>) -> impl Iterator<Item = ElementRef<'a>> {
    let page = el.page();
    el.node()
        .children
        .iter()
        .map(move |&index| page.get(index))
        .filter(|child| !child.node().is_text())
}

fn is_custom_element(tag: &str) -> bool {
    tag.contains('-')
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn implicit_of(html: &str, tag: &str) -> Option<&'static str> {
        let page = page_from_html(html);
        let el = page.by_tag(tag).next().unwrap();
        implicit_role(el).map(|role| role.name)
    }

    #[rstest]
    #[case(r#"<a href="/">x</a>"#, "a", Some("link"))]
    #[case("<a>x</a>", "a", Some("generic"))]
    #[case(r#"<img src="a.png" alt="A cat">"#, "img", Some("img"))]
    #[case(r#"<img src="a.png">"#, "img", Some("img"))]
    #[case(r#"<img src="a.png" alt="">"#, "img", Some("none"))]
    #[case(r#"<img src="a.png" alt="" aria-label="A cat">"#, "img", Some("img"))]
    #[case("<input>", "input", Some("textbox"))]
    #[case(r#"<input type="BOGUS">"#, "input", Some("textbox"))]
    #[case(r#"<input type="email" list="l">"#, "input", Some("combobox"))]
    #[case(r#"<input type="search">"#, "input", Some("searchbox"))]
    #[case(r#"<input type="checkbox">"#, "input", Some("checkbox"))]
    #[case(r#"<input type="range">"#, "input", Some("slider"))]
    #[case(r#"<input type="submit">"#, "input", Some("button"))]
    #[case(r#"<input type="password">"#, "input", None)]
    #[case("<section>x</section>", "section", Some("generic"))]
    #[case(r#"<section aria-label="News">x</section>"#, "section", Some("region"))]
    #[case("<body><header>x</header></body>", "header", Some("banner"))]
    #[case("<article><header>x</header></article>", "header", Some("generic"))]
    #[case(
        r#"<div role="main"><footer>x</footer></div>"#,
        "footer",
        Some("generic")
    )]
    #[case("<footer>x</footer>", "footer", Some("contentinfo"))]
    #[case("<select><option>a</option></select>", "select", Some("combobox"))]
    #[case(
        "<select multiple><option>a</option></select>",
        "select",
        Some("listbox")
    )]
    #[case(
        r#"<select size="4"><option>a</option></select>"#,
        "select",
        Some("listbox")
    )]
    #[case("<select><option>a</option></select>", "option", Some("option"))]
    #[case("<ul><li>a</li></ul>", "li", Some("listitem"))]
    #[case(r#"<ul role="tablist"><li>a</li></ul>"#, "li", Some("generic"))]
    #[case("<table><tr><td>a</td></tr></table>", "td", Some("cell"))]
    #[case(
        r#"<table role="grid"><tr><td>a</td></tr></table>"#,
        "td",
        Some("gridcell")
    )]
    #[case(
        r#"<table role="presentation"><tr><td>a</td></tr></table>"#,
        "td",
        None
    )]
    #[case("<table><tr><th>h</th></tr></table>", "th", Some("columnheader"))]
    #[case(
        "<table><tr><th>h</th><td>a</td></tr></table>",
        "th",
        Some("rowheader")
    )]
    #[case(
        r#"<table><tr><th scope="col">h</th><td>a</td></tr></table>"#,
        "th",
        Some("columnheader")
    )]
    #[case("<h3>x</h3>", "h3", Some("heading"))]
    #[case("<div>x</div>", "div", Some("generic"))]
    #[case("<abbr>x</abbr>", "abbr", None)]
    #[case("<my-widget>x</my-widget>", "my-widget", Some("generic"))]
    fn implicit_roles(#[case] html: &str, #[case] tag: &str, #[case] expected: Option<&str>) {
        assert_eq!(implicit_of(html, tag), expected);
    }

    fn aria_of(html: &str, tag: &str) -> AllowedAria {
        let page = page_from_html(html);
        allowed_aria(page.by_tag(tag).next().unwrap())
    }

    #[test]
    fn allowed_aria_follows_the_conditional_rows() {
        assert_eq!(aria_of("<script></script>", "script"), AllowedAria::NoAria);
        assert_eq!(
            aria_of(r#"<input type="hidden">"#, "input"),
            AllowedAria::NoAria
        );
        assert_eq!(aria_of("<br>", "br"), AllowedAria::AriaHiddenOnly);
        assert_eq!(
            aria_of(r#"<img src="a" alt="">"#, "img"),
            AllowedAria::AriaHiddenTrueOnly
        );
        assert_eq!(
            aria_of("<legend>x</legend>", "legend"),
            AllowedAria::GlobalOnly
        );
        assert_eq!(
            aria_of("<dl><dd>x</dd></dl>", "dd"),
            AllowedAria::GlobalAndRole("definition")
        );
        assert_eq!(
            aria_of(r#"<input type="date">"#, "input"),
            AllowedAria::GlobalAndRole("textbox")
        );
        assert_eq!(
            aria_of(r#"<input type="file">"#, "input"),
            AllowedAria::GlobalAnd(&["aria-disabled", "aria-invalid", "aria-required"])
        );
        assert_eq!(
            aria_of("<button>x</button>", "button"),
            AllowedAria::GlobalAndEffectiveRole
        );
    }
}
