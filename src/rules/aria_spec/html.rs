//! ARIA in HTML (W3C Recommendation of 11 August 2026, w3c/html-aria commit
//! `e277aa3`): each element's implicit role, the roles and `aria-*`
//! attributes authors may set on it, and the native attributes with an
//! `aria-*` equivalent. Conditional rows ("`a` with `href`", "`img` with
//! `alt=""`", ...) are resolved against the element.

use crate::page::ElementRef;

use super::element::{effective_role, first_valid_role};
use super::roles::{Role, role};

/// "No `role` or `aria-*` attributes" in the document conformance table
/// (plus `input type=hidden`, which [`allowed_aria`] handles).
pub const NO_ROLE_OR_ARIA_ELEMENTS: &[&str] = &[
    "base", "col", "colgroup", "head", "link", "map", "meta", "noscript", "param", "script",
    "slot", "source", "style", "template", "title", "track",
];

/// Roles ARIA in HTML asks conformance checkers to warn about: WAI-ARIA's
/// deprecated `directory` and DPub-ARIA's deprecated ones.
pub const DEPRECATED_ROLES: &[&str] = &["directory", "doc-biblioentry", "doc-endnote"];

/// A native attribute and the `aria-*` attribute with the same implicit
/// semantics, on the elements that allow the native one (`None`: any).
#[derive(Debug)]
pub struct NativeEquivalent {
    pub native: &'static str,
    pub aria: &'static str,
    pub elements: Option<&'static [&'static str]>,
}

pub const NATIVE_EQUIVALENTS: &[NativeEquivalent] = &[
    NativeEquivalent {
        native: "checked",
        aria: "aria-checked",
        elements: Some(&["input"]),
    },
    NativeEquivalent {
        native: "disabled",
        aria: "aria-disabled",
        elements: Some(&[
            "button", "fieldset", "input", "optgroup", "option", "select", "textarea",
        ]),
    },
    NativeEquivalent {
        native: "hidden",
        aria: "aria-hidden",
        elements: None,
    },
    NativeEquivalent {
        native: "placeholder",
        aria: "aria-placeholder",
        elements: Some(&["input", "textarea"]),
    },
    NativeEquivalent {
        native: "max",
        aria: "aria-valuemax",
        elements: Some(&["input", "meter", "progress"]),
    },
    NativeEquivalent {
        native: "min",
        aria: "aria-valuemin",
        elements: Some(&["input", "meter"]),
    },
    NativeEquivalent {
        native: "readonly",
        aria: "aria-readonly",
        elements: Some(&["input", "textarea"]),
    },
    NativeEquivalent {
        native: "contenteditable",
        aria: "aria-readonly",
        elements: None,
    },
    NativeEquivalent {
        native: "required",
        aria: "aria-required",
        elements: Some(&["input", "select", "textarea"]),
    },
    NativeEquivalent {
        native: "colspan",
        aria: "aria-colspan",
        elements: Some(&["td", "th"]),
    },
    NativeEquivalent {
        native: "rowspan",
        aria: "aria-rowspan",
        elements: Some(&["td", "th"]),
    },
];

/// The roles an author may set on an element. Its implicit role is always
/// allowed too, but NOT RECOMMENDED, so it isn't listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedRoles {
    Any,
    Only(&'static [&'static str]),
    NoRole,
}

impl AllowedRoles {
    pub fn allows(&self, role: &str) -> bool {
        match self {
            AllowedRoles::Any => true,
            AllowedRoles::Only(roles) => roles.contains(&role),
            AllowedRoles::NoRole => false,
        }
    }
}

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

const A_WITH_HREF_ROLES: &[&str] = &[
    "button",
    "checkbox",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "switch",
    "tab",
    "treeitem",
    "doc-backlink",
    "doc-biblioref",
    "doc-glossref",
    "doc-noteref",
];
const ARTICLE_ROLES: &[&str] = &[
    "application",
    "document",
    "feed",
    "main",
    "none",
    "presentation",
    "region",
];
const ASIDE_ROLES: &[&str] = &[
    "feed",
    "none",
    "note",
    "presentation",
    "region",
    "search",
    "doc-dedication",
    "doc-example",
    "doc-footnote",
    "doc-glossary",
    "doc-pullquote",
    "doc-tip",
];
const BUTTON_ROLES: &[&str] = &[
    "checkbox",
    "combobox",
    "gridcell",
    "link",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "separator",
    "slider",
    "switch",
    "tab",
    "treeitem",
];
const INPUT_IMAGE_ROLES: &[&str] = &[
    "button",
    "checkbox",
    "gridcell",
    "link",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "separator",
    "slider",
    "switch",
    "tab",
    "treeitem",
];
const INPUT_RESET_SUBMIT_ROLES: &[&str] = &[
    "button",
    "checkbox",
    "combobox",
    "gridcell",
    "link",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "separator",
    "slider",
    "switch",
    "tab",
    "treeitem",
];
const EMBED_ROLES: &[&str] = &[
    "application",
    "document",
    "img",
    "image",
    "none",
    "presentation",
];
const IMG_WITH_NAME_ROLES: &[&str] = &[
    "button",
    "checkbox",
    "link",
    "math",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "meter",
    "option",
    "progressbar",
    "radio",
    "scrollbar",
    "separator",
    "slider",
    "switch",
    "tab",
    "treeitem",
    "doc-cover",
];
const LIST_ROLES: &[&str] = &[
    "group",
    "listbox",
    "menu",
    "menubar",
    "none",
    "presentation",
    "radiogroup",
    "tablist",
    "toolbar",
    "tree",
];
const NAV_ROLES: &[&str] = &[
    "menu",
    "menubar",
    "none",
    "presentation",
    "tablist",
    "doc-index",
    "doc-pagelist",
    "doc-toc",
];
const SECTION_ROLES: &[&str] = &[
    "alert",
    "alertdialog",
    "application",
    "banner",
    "complementary",
    "contentinfo",
    "dialog",
    "document",
    "feed",
    "group",
    "log",
    "main",
    "marquee",
    "navigation",
    "none",
    "note",
    "presentation",
    "search",
    "status",
    "tabpanel",
    "doc-abstract",
    "doc-acknowledgments",
    "doc-afterword",
    "doc-appendix",
    "doc-bibliography",
    "doc-chapter",
    "doc-colophon",
    "doc-conclusion",
    "doc-credit",
    "doc-credits",
    "doc-dedication",
    "doc-endnotes",
    "doc-epigraph",
    "doc-epilogue",
    "doc-errata",
    "doc-example",
    "doc-foreword",
    "doc-glossary",
    "doc-index",
    "doc-introduction",
    "doc-notice",
    "doc-pagelist",
    "doc-part",
    "doc-preface",
    "doc-prologue",
    "doc-pullquote",
    "doc-qna",
    "doc-toc",
];
const PRESENTATIONAL_ROLES: &[&str] = &["none", "presentation"];

const NAMING_PROHIBITED_ELEMENTS: &[&str] = &[
    "abbr",
    "b",
    "bdi",
    "bdo",
    "body",
    "caption",
    "cite",
    "code",
    "data",
    "del",
    "div",
    "em",
    "figcaption",
    "i",
    "ins",
    "kbd",
    "label",
    "legend",
    "mark",
    "p",
    "pre",
    "q",
    "rp",
    "rt",
    "s",
    "samp",
    "selectedcontent",
    "small",
    "span",
    "strong",
    "sub",
    "sup",
    "time",
    "u",
    "var",
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

pub fn allowed_roles(el: ElementRef) -> AllowedRoles {
    use AllowedRoles::{Any, NoRole, Only};
    match el.tag() {
        "a" if el.has_attr("href") => Only(A_WITH_HREF_ROLES),
        "area" if el.has_attr("href") => NoRole,
        "area" => Only(&["button", "link"]),
        "article" => Only(ARTICLE_ROLES),
        "aside" => Only(ASIDE_ROLES),
        "audio" | "video" => Only(&["application"]),
        "br" | "wbr" => Only(PRESENTATIONAL_ROLES),
        "button" if is_first_child_of_select(el) => NoRole,
        "button" => Only(BUTTON_ROLES),
        "dialog" => Only(&["alertdialog"]),
        "div" if el.parent().is_some_and(|parent| parent.tag() == "dl") => {
            Only(PRESENTATIONAL_ROLES)
        }
        "dl" => Only(&["group", "list", "none", "presentation"]),
        "dt" => Only(&["listitem"]),
        "embed" | "iframe" => Only(EMBED_ROLES),
        "fieldset" => Only(&["none", "presentation", "radiogroup"]),
        "figcaption" => Only(&["group", "none", "presentation"]),
        "figure" if el.descendants().any(|d| d.tag() == "figcaption") => Only(&["doc-example"]),
        "footer" => Only(&["group", "none", "presentation", "doc-footnote"]),
        "form" => Only(&["none", "presentation", "search"]),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            Only(&["none", "presentation", "tab", "doc-subtitle"])
        }
        "header" => Only(&["group", "none", "presentation"]),
        "hr" => Only(&["none", "presentation", "doc-pagebreak"]),
        "html" => Only(&["document"]),
        "img" => img_allowed_roles(el),
        "input" => input_allowed_roles(el),
        "label" if is_associated_label(el) => NoRole,
        "li" if parent_is_list(el) => NoRole,
        "menu" | "ol" | "ul" => Only(LIST_ROLES),
        "nav" => Only(NAV_ROLES),
        "object" => Only(&["application", "document", "img", "image"]),
        "search" => Only(&["form", "group", "none", "presentation", "region"]),
        "section" => Only(SECTION_ROLES),
        "select" if is_list_box_select(el) => NoRole,
        "select" => Only(&["menu"]),
        "selectedcontent" if has_ancestor(el, "select") => NoRole,
        "summary" if is_details_summary(el) => NoRole,
        "td" | "th" | "tr" if table_role(el).is_some() => NoRole,
        "caption" | "datalist" | "dd" | "details" | "legend" | "main" | "math" | "meter"
        | "optgroup" | "option" | "picture" | "progress" | "textarea" | "body" => NoRole,
        tag if NO_ROLE_OR_ARIA_ELEMENTS.contains(&tag) => NoRole,
        _ => Any,
    }
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

/// Whether the element's implicit semantics make it "Naming Prohibited":
/// no `aria-label`/`aria-labelledby` unless an allowed explicit role that
/// can be named overrides them.
pub fn naming_prohibited(el: ElementRef) -> bool {
    match el.tag() {
        "a" | "area" => !el.has_attr("href"),
        "header" | "footer" => is_scoped_to_sectioning_content(el),
        tag if is_custom_element(tag) => true,
        tag => NAMING_PROHIBITED_ELEMENTS.contains(&tag),
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

fn input_allowed_roles(el: ElementRef) -> AllowedRoles {
    use AllowedRoles::{NoRole, Only};
    match input_type(el) {
        "button" => Only(BUTTON_ROLES),
        "checkbox" if el.has_attr("aria-pressed") => {
            Only(&["menuitemcheckbox", "option", "switch", "button"])
        }
        "checkbox" => Only(&["menuitemcheckbox", "option", "switch"]),
        "image" => Only(INPUT_IMAGE_ROLES),
        "reset" | "submit" => Only(INPUT_RESET_SUBMIT_ROLES),
        "radio" => Only(&["menuitemradio"]),
        "text" if !el.has_attr("list") => Only(&["combobox", "searchbox", "spinbutton"]),
        _ => NoRole,
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

fn img_allowed_roles(el: ElementRef) -> AllowedRoles {
    match (is_decorative_img(el), el.attr("alt"), has_author_name(el)) {
        (true, _, _) => AllowedRoles::NoRole,
        (_, Some(alt), _) if !alt.trim().is_empty() => AllowedRoles::Only(IMG_WITH_NAME_ROLES),
        (_, _, true) => AllowedRoles::Only(IMG_WITH_NAME_ROLES),
        _ => AllowedRoles::Only(PRESENTATIONAL_ROLES),
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

fn is_associated_label(label: ElementRef) -> bool {
    let target = label
        .attr("for")
        .and_then(|id| label.page().element_by_id(id));
    match target {
        Some(target) => is_labelable(target),
        None if label.has_attr("for") => false,
        None => label.descendants().any(is_labelable),
    }
}

fn is_labelable(el: ElementRef) -> bool {
    match el.tag() {
        "input" => input_type(el) != "hidden",
        tag => matches!(
            tag,
            "button" | "meter" | "output" | "progress" | "select" | "textarea"
        ),
    }
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
    use crate::rules::aria_spec::attributes::attribute;
    use crate::rules::aria_spec::roles::concrete_role;

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

    fn allowed_of(html: &str, tag: &str) -> AllowedRoles {
        let page = page_from_html(html);
        allowed_roles(page.by_tag(tag).next().unwrap())
    }

    #[test]
    fn allowed_roles_follow_the_conditional_rows() {
        assert!(allowed_of(r#"<a href="/">x</a>"#, "a").allows("tab"));
        assert!(!allowed_of(r#"<a href="/">x</a>"#, "a").allows("heading"));
        assert_eq!(allowed_of("<a>x</a>", "a"), AllowedRoles::Any);
        assert_eq!(
            allowed_of(r#"<area href="/">"#, "area"),
            AllowedRoles::NoRole
        );
        assert_eq!(
            allowed_of(r#"<img src="a" alt="">"#, "img"),
            AllowedRoles::NoRole
        );
        assert!(allowed_of(r#"<img src="a" alt="A">"#, "img").allows("button"));
        assert!(allowed_of(r#"<img src="a">"#, "img").allows("presentation"));
        assert!(!allowed_of(r#"<img src="a">"#, "img").allows("button"));
        assert!(allowed_of("<embed>", "embed").allows("image"));
        assert_eq!(
            allowed_of(r#"<input type="email">"#, "input"),
            AllowedRoles::NoRole
        );
        assert!(!allowed_of(r#"<input type="checkbox">"#, "input").allows("button"));
        assert!(
            allowed_of(r#"<input type="checkbox" aria-pressed="true">"#, "input").allows("button")
        );
        assert!(allowed_of("<dl><div>x</div></dl>", "div").allows("none"));
        assert!(!allowed_of("<dl><div>x</div></dl>", "div").allows("button"));
        assert_eq!(
            allowed_of("<ul><li>a</li></ul>", "li"),
            AllowedRoles::NoRole
        );
        assert_eq!(allowed_of("<div><li>a</li></div>", "li"), AllowedRoles::Any);
        assert_eq!(
            allowed_of(r#"<label for="i">L</label><input id="i">"#, "label"),
            AllowedRoles::NoRole
        );
        assert_eq!(allowed_of("<label>L</label>", "label"), AllowedRoles::Any);
        assert_eq!(
            allowed_of("<details><summary>s</summary></details>", "summary"),
            AllowedRoles::NoRole
        );
        assert_eq!(
            allowed_of("<select><button>b</button></select>", "button"),
            AllowedRoles::NoRole
        );
        assert_eq!(
            allowed_of("<meta charset=utf-8>", "meta"),
            AllowedRoles::NoRole
        );
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

    #[rstest]
    #[case("<span>x</span>", "span", true)]
    #[case("<a>x</a>", "a", true)]
    #[case(r#"<a href="/">x</a>"#, "a", false)]
    #[case("<article><footer>x</footer></article>", "footer", true)]
    #[case("<footer>x</footer>", "footer", false)]
    #[case("<button>x</button>", "button", false)]
    fn naming_prohibited_elements(#[case] html: &str, #[case] tag: &str, #[case] expected: bool) {
        let page = page_from_html(html);
        assert_eq!(
            naming_prohibited(page.by_tag(tag).next().unwrap()),
            expected
        );
    }

    #[test]
    fn every_listed_role_is_known() {
        let lists = [
            A_WITH_HREF_ROLES,
            ARTICLE_ROLES,
            ASIDE_ROLES,
            BUTTON_ROLES,
            INPUT_IMAGE_ROLES,
            INPUT_RESET_SUBMIT_ROLES,
            EMBED_ROLES,
            IMG_WITH_NAME_ROLES,
            LIST_ROLES,
            NAV_ROLES,
            SECTION_ROLES,
            PRESENTATIONAL_ROLES,
        ];
        for name in lists
            .into_iter()
            .flatten()
            .filter(|name| !name.starts_with("doc-"))
        {
            assert!(concrete_role(name).is_some(), "{name}");
        }
    }

    #[test]
    fn native_equivalents_name_real_aria_attributes() {
        for equivalent in NATIVE_EQUIVALENTS {
            assert!(attribute(equivalent.aria).is_some());
        }
    }
}
