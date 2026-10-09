//! ACT rules for the page title and for `lang` attributes.

use super::language_subtags::LANGUAGE_SUBTAGS;
use crate::page::{ElementRef, RenderedPage};
use crate::rules::Finding;
use crate::rules::aria_spec::is_not_rendered;
use crate::rules::markup::start_tag_first;

/// 2779a5: the first HTML `title` element must contain text.
pub fn check_page_title(page: &RenderedPage) -> Vec<Finding> {
    let message = match first_title(page).map(|title| title.text()) {
        Some(text) if !text.is_empty() => return Vec::new(),
        Some(_) => "the page's first <title> element is empty",
        None => "the page has no <title> element",
    };
    vec![Finding::error("2779a5", message).help(
        "put a <title> with text in the <head>, e.g. <title>Order summary - Acme</title>; \
         only the first <title> counts",
    )]
}

/// An `svg` has its own `title` elements, and `template` content isn't in
/// the document.
fn first_title(page: &RenderedPage) -> Option<ElementRef<'_>> {
    page.by_tag("title").find(|title| {
        !title
            .ancestors()
            .any(|ancestor| matches!(ancestor.tag(), "svg" | "template"))
    })
}

/// b5c3f8: the `html` element needs a `lang` attribute that isn't empty or
/// only whitespace.
pub fn check_html_lang(page: &RenderedPage) -> Vec<Finding> {
    let Some(html) = page.by_tag("html").next() else {
        return Vec::new();
    };
    let message = match html.attr("lang") {
        Some(lang) if !lang.trim().is_empty() => return Vec::new(),
        Some(lang) => format!("<html lang=\"{lang}\"> declares no language"),
        None => "<html> has no lang attribute".to_string(),
    };
    vec![
        Finding::error("b5c3f8", message)
            .help("declare the page's main language, e.g. <html lang=\"en\">"),
    ]
}

/// bf051a: the `html` element's `lang` must start with a primary language
/// subtag from the IANA registry.
pub fn check_html_lang_valid(page: &RenderedPage) -> Vec<Finding> {
    page.by_tag("html")
        .next()
        .and_then(|html| Some((html, html.non_empty_attr("lang")?)))
        .filter(|(_, lang)| !has_known_primary_language(lang))
        .map(|(_, lang)| {
            Finding::error(
                "bf051a",
                format!("<html lang=\"{lang}\"> has no known primary language subtag"),
            )
            .help(unknown_language_help(lang))
        })
        .into_iter()
        .collect()
}

/// de46e4: every `lang` in the `body` on an element whose text inherits it
/// must start with a known primary language subtag.
pub fn check_element_lang_valid(page: &RenderedPage) -> Vec<Finding> {
    page.all()
        .filter_map(|el| Some((el, el.attr("lang")?)))
        .filter(|(el, lang)| !lang.is_empty() && is_in_body(*el))
        .filter(|(el, _)| has_inheriting_text(*el))
        .filter(|(_, lang)| !has_known_primary_language(lang))
        .map(|(el, lang)| {
            Finding::error_at(
                el,
                "de46e4",
                format!(
                    "{} has no known primary language subtag",
                    start_tag_first(el, &["lang"])
                ),
            )
            .help(unknown_language_help(lang))
        })
        .collect()
}

fn is_in_body(el: ElementRef) -> bool {
    el.self_and_ancestors().any(|el| el.tag() == "body")
}

/// Text whose nearest `lang` is this element's: text nodes, and image
/// `alt` text, outside any descendant that sets its own `lang`.
fn has_inheriting_text(el: ElementRef) -> bool {
    el.has_own_text()
        || el.element_children().any(|child| {
            !child.has_attr("lang")
                && !is_not_rendered(child)
                && (has_alt_text(child) || has_inheriting_text(child))
        })
}

fn has_alt_text(el: ElementRef) -> bool {
    el.tag() == "img" && el.non_empty_attr("alt").is_some()
}

/// RFC 5646's first subtag, matched case-insensitively. Grandfathered tags
/// (`i-lux`) and ISO 639-2 codes with a two-letter equivalent (`eng`) aren't
/// in the registry, so they fail too.
fn has_known_primary_language(lang: &str) -> bool {
    let primary = lang
        .trim_matches(|c: char| c.is_ascii_whitespace())
        .split('-')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    LANGUAGE_SUBTAGS.binary_search(&primary.as_str()).is_ok() || is_private_use(&primary)
}

/// The registry's `qaa..qtz` range.
fn is_private_use(subtag: &str) -> bool {
    subtag.len() == 3
        && subtag.chars().all(|c| c.is_ascii_lowercase())
        && ("qaa"..="qtz").contains(&subtag)
}

fn unknown_language_help(lang: &str) -> String {
    let trimmed = lang.trim();
    match trimmed.len() {
        0 => "set it to a language code, e.g. lang=\"en\", or remove the attribute".to_string(),
        _ => format!(
            "start it with a two-letter (or registered three-letter) language code, \
             e.g. lang=\"en\" or lang=\"fr-CH\", instead of \"{trimmed}\""
        ),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::count;

    #[rstest]
    #[case("<html><h1>this page has no title</h1></html>", 1)]
    #[case("<html><title></title></html>", 1)]
    #[case("<html><title> </title></html>", 1)]
    #[case(
        "<html><head><title></title></head><body><title>Title</title></body></html>",
        1
    )]
    #[case("<html><body><svg><title>Icon</title></svg></body></html>", 1)]
    #[case("<html><title>This page has a title</title></html>", 0)]
    #[case("<html><body><title>Title of the page.</title></body></html>", 0)]
    #[case(
        "<html><head><title>Title</title></head><body><title></title></body></html>",
        0
    )]
    fn page_title_2779a5(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_page_title, html), expected, "{html}");
    }

    #[rstest]
    #[case("<html><body>The quick brown fox</body></html>", 1)]
    #[case(r#"<html lang=""><body>x</body></html>"#, 1)]
    #[case(r#"<html lang=" "><body>x</body></html>"#, 1)]
    #[case(r#"<html xml:lang="en"><body>x</body></html>"#, 1)]
    #[case(r#"<html lang="en"><body>x</body></html>"#, 0)]
    fn html_lang_b5c3f8(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_html_lang, html), expected, "{html}");
    }

    #[rstest]
    #[case(r#"<html lang="em-US"></html>"#, 1)]
    #[case(r##"<html lang="#1"></html>"##, 1)]
    #[case(r#"<html lang="eng"></html>"#, 1)]
    #[case(r#"<html lang="i-lux"></html>"#, 1)]
    #[case(r#"<html lang="FR"></html>"#, 0)]
    #[case(r#"<html lang="en-US-GB"></html>"#, 0)]
    #[case(r#"<html lang="yue-HK"></html>"#, 0)]
    #[case(r#"<html lang="qab"></html>"#, 0)]
    #[case(r#"<html lang=" "></html>"#, 0)]
    #[case("<html></html>", 0)]
    fn html_lang_valid_bf051a(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_html_lang_valid, html), expected, "{html}");
    }

    #[rstest]
    #[case(
        r#"<html lang="es"><body><article lang="dutch">Zij liepen</article></body></html>"#,
        1
    )]
    #[case(
        r##"<html lang="en"><body><article lang="#!">They wandered</article></body></html>"##,
        1
    )]
    #[case(
        r#"<html lang="fr"><body><article lang="  ">They wandered</article></body></html>"#,
        1
    )]
    #[case(r#"<html lang="es"><body><article lang="english"><p aria-hidden="true">They</p></article></body></html>"#, 1)]
    #[case(r#"<html lang="es"><body><article lang="en"><div lang="invalid">They</div></article></body></html>"#, 1)]
    #[case(r#"<html lang="en"><body><div lang="invalid"><img src="f.jpg" alt="Fireworks"></div></body></html>"#, 1)]
    #[case(
        r#"<html lang="en"><body><p lang="eng">I love ACT rules!</p></body></html>"#,
        1
    )]
    #[case(
        r#"<html lang="es"><body><article lang="en">They wandered</article></body></html>"#,
        0
    )]
    #[case(
        r#"<html lang="en"><body><blockquote lang="fr-CH">Ils ont</blockquote></body></html>"#,
        0
    )]
    #[case(
        r#"<html lang="fr"><body><p lang="en-US-GB">They wandered</p></body></html>"#,
        0
    )]
    #[case(r#"<html lang="fr"><body><article lang="invalid"><div lang="en">They</div></article></body></html>"#, 0)]
    #[case(r#"<html lang="en"><body>They wandered</body></html>"#, 0)]
    #[case(
        r#"<html lang="en"><body><article lang="">They wandered</article></body></html>"#,
        0
    )]
    #[case(
        r#"<html lang="en"><body><p lang="hidden"><span hidden>They</span></p></body></html>"#,
        0
    )]
    #[case(
        r#"<html lang="en"><body><div lang="invalid"><img src="f.jpg" alt=""></div></body></html>"#,
        0
    )]
    #[case(r#"<html lang="en"><body><div lang="invalid"></div></body></html>"#, 0)]
    #[case(r#"<html lang="xx"><body>Only the html element</body></html>"#, 0)]
    fn element_lang_valid_de46e4(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_element_lang_valid, html), expected, "{html}");
    }

    #[test]
    fn subtag_list_is_sorted_for_binary_search() {
        assert!(LANGUAGE_SUBTAGS.windows(2).all(|pair| pair[0] < pair[1]));
    }
}
