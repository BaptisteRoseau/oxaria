//! WCAG 2.4.7 check for visible focus indicators declared in `style` tags.
//!
//! litehtml can't simulate `:focus`, so this scans the raw stylesheet text instead. Rules nested in
//! `@media`/`@supports` blocks are not inspected.

use crate::page::RenderedPage;

use crate::rules::{CheckOptions, Finding};

/// G195: removing the focus outline (`outline: none`/`0`) without declaring another visible
/// indicator (border, box-shadow, background) leaves keyboard users unable to see where focus is.
pub fn check_outline_removed_without_alternative(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    split_css_rules(&strip_comments(&page.stylesheet_text()))
        .into_iter()
        .filter(|(selector, _)| selector.to_lowercase().contains(":focus"))
        .filter(|(_, body)| removes_outline_without_alternative(body))
        .map(|(selector, _)| {
            Finding::error("G195", outline_removed_message(&selector))
                .help(outline_removed_help(&selector))
        })
        .collect()
}

fn outline_removed_message(selector: &str) -> String {
    format!("\"{selector}\" removes the focus outline without another visible focus indicator")
}

fn outline_removed_help(selector: &str) -> String {
    format!(
        "remove \"outline: none\" from \"{selector}\", or replace it with another visible \
         indicator, e.g. box-shadow: 0 0 0 3px #1a73e8"
    )
}

fn removes_outline_without_alternative(body: &str) -> bool {
    declares_no_outline(body) && !declares_alternative_indicator(body)
}

fn declares_no_outline(body: &str) -> bool {
    property_value(body, "outline")
        .or_else(|| property_value(body, "outline-style"))
        .is_some_and(is_none_value)
}

fn is_none_value(value: &str) -> bool {
    matches!(value.trim(), "none" | "0" | "0px")
}

fn declares_alternative_indicator(body: &str) -> bool {
    ["box-shadow", "border", "background", "background-color"]
        .iter()
        .any(|property| property_value(body, property).is_some())
}

fn property_value<'a>(body: &'a str, property: &str) -> Option<&'a str> {
    body.split(';').find_map(|declaration| {
        let (name, value) = declaration.split_once(':')?;
        (name.trim().eq_ignore_ascii_case(property)).then(|| value.trim())
    })
}

fn strip_comments(css: &str) -> String {
    let mut stripped = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        stripped.push_str(&rest[..start]);
        let comment = &rest[start + 2..];
        rest = comment.find("*/").map_or("", |end| &comment[end + 2..]);
    }
    stripped.push_str(rest);
    stripped
}

/// Splits a stylesheet into top-level `(selector, declaration_body)` pairs, tracking brace depth
/// so a nested at-rule block is captured whole rather than mis-split.
fn split_css_rules(css: &str) -> Vec<(String, String)> {
    let mut rules = Vec::new();
    let mut depth = 0i32;
    let mut selector_start = 0usize;
    let mut body_start = 0usize;
    for (index, byte) in css.bytes().enumerate() {
        match byte {
            b'{' => {
                if depth == 0 {
                    body_start = index + 1;
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    rules.push((
                        css[selector_start..body_start - 1].trim().to_string(),
                        css[body_start..index].to_string(),
                    ));
                    selector_start = index + 1;
                }
            }
            _ => {}
        }
    }
    rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn split_css_rules_splits_simple_stylesheet() {
        let rules = split_css_rules("a{color:red} b{color:blue}");
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].0, "a");
        assert_eq!(rules[0].1, "color:red");
    }

    #[test]
    fn split_css_rules_keeps_media_block_whole() {
        let rules = split_css_rules("@media (max-width:600px){ a:focus{outline:none} }");
        assert_eq!(rules.len(), 1);
        assert!(rules[0].0.starts_with("@media"));
    }

    #[test]
    fn strip_comments_removes_every_comment() {
        assert_eq!(
            strip_comments("a{/* } */x:y}/* b */c{}/* open"),
            "a{x:y}c{}"
        );
    }

    #[test]
    fn braces_in_comments_do_not_hide_later_rules() {
        let p = page_from_html(
            "<style>/* legacy } */ a { color: red } a:focus { outline: none; }</style>",
        );
        let findings = check_outline_removed_without_alternative(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn outline_none_without_alternative_is_flagged() {
        let p = page_from_html("<style>a:focus, button:focus { outline: none; }</style>");
        let findings = check_outline_removed_without_alternative(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "G195");
    }

    #[test]
    fn outline_none_with_box_shadow_alternative_is_not_flagged() {
        let p = page_from_html(
            "<style>a:focus-visible { outline: none; box-shadow: 0 0 0 3px blue; }</style>",
        );
        assert!(check_outline_removed_without_alternative(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn rule_without_focus_selector_is_ignored() {
        let p = page_from_html("<style>a { outline: none; }</style>");
        assert!(check_outline_removed_without_alternative(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn no_style_tag_is_not_flagged() {
        let p = page_from_html("<body></body>");
        assert!(check_outline_removed_without_alternative(&p, &CheckOptions::default()).is_empty());
    }
}
