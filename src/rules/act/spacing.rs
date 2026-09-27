//! ACT rules for text spacing locked with `!important` in a `style`
//! attribute, which users' own text-spacing styles can't override.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::is_not_rendered;
use crate::rules::wcag22::renders_own_text;
use crate::rules::{CheckOptions, Finding};

const POINTS_TO_PIXELS: f64 = 4.0 / 3.0;
/// Absorbs float error, so that a spacing of exactly the minimum passes.
const TOLERANCE: f64 = 1e-6;

struct SpacingRule {
    id: &'static str,
    property: &'static str,
    minimum: f64,
}

const LETTER_SPACING: SpacingRule = SpacingRule {
    id: "24afc2",
    property: "letter-spacing",
    minimum: 0.12,
};

const WORD_SPACING: SpacingRule = SpacingRule {
    id: "9e45ec",
    property: "word-spacing",
    minimum: 0.16,
};

/// 24afc2: `letter-spacing` set with `!important` in a `style` attribute
/// must be at least 0.12 times the font size.
pub fn check_important_letter_spacing(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    spacing_findings(page, &LETTER_SPACING)
}

/// 9e45ec: `word-spacing` set with `!important` in a `style` attribute must
/// be at least 0.16 times the font size.
pub fn check_important_word_spacing(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    spacing_findings(page, &WORD_SPACING)
}

fn spacing_findings(page: &RenderedPage, rule: &SpacingRule) -> Vec<Finding> {
    page.all()
        .filter(|el| renders_own_text(*el) && !is_not_rendered(*el))
        .filter_map(|el| Some((el, important_value(el.attr("style")?, rule.property)?)))
        .filter_map(|(el, value)| Some((el, spacing_ratio(el, &value)?, value)))
        .filter(|(_, ratio, _)| *ratio + TOLERANCE < rule.minimum)
        .map(|(el, ratio, value)| {
            Finding::error(
                rule.id,
                format!(
                    "<{}> locks {} at \"{value} !important\" ({ratio:.2} times the font size, \
                     below {:.2})",
                    el.tag(),
                    rule.property,
                    rule.minimum
                ),
            )
            .at(el)
            .help(format!(
                "drop !important so users' text-spacing styles can override it, or use \
                 {}: {}em or more",
                rule.property, rule.minimum
            ))
        })
        .collect()
}

/// The value of the declaration that wins the cascade within the `style`
/// attribute, when it is `!important`: the last important one.
fn important_value(style: &str, property: &str) -> Option<String> {
    style
        .split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .filter(|(name, _)| name.trim().eq_ignore_ascii_case(property))
        .filter_map(|(_, value)| strip_important(value))
        .next_back()
}

fn strip_important(value: &str) -> Option<String> {
    let (value, flag) = value.rsplit_once('!')?;
    flag.trim()
        .eq_ignore_ascii_case("important")
        .then(|| value.trim().to_ascii_lowercase())
}

/// The spacing as a multiple of the element's font size. `None` for values
/// whose computed value is inherited, so not important (`inherit`,
/// `unset`), and for units that can't be resolved here.
fn spacing_ratio(el: ElementRef, value: &str) -> Option<f64> {
    let font_size = f64::from(el.font_size_px());
    match value {
        "normal" | "initial" => Some(0.0),
        _ => {
            let (number, unit) = split_length(value)?;
            match unit {
                "em" => Some(number),
                "px" => Some(number / font_size),
                "pt" => Some(number * POINTS_TO_PIXELS / font_size),
                "rem" => Some(number * root_font_size(el) / font_size),
                "" if number == 0.0 => Some(0.0),
                _ => None,
            }
        }
    }
}

fn split_length(value: &str) -> Option<(f64, &str)> {
    let unit_start = value
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+')))
        .unwrap_or(value.len());
    let (number, unit) = value.split_at(unit_start);
    Some((number.parse().ok()?, unit.trim()))
}

fn root_font_size(el: ElementRef) -> f64 {
    let root = el.ancestors().last().unwrap_or(el);
    f64::from(root.font_size_px())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;
    use crate::rules::RuleCheck;

    const TEXT: &str = "The toy brought back fond memories.";

    fn count(check: RuleCheck, html: &str) -> usize {
        check(&page_from_html(html), &CheckOptions::default()).len()
    }

    #[rstest]
    #[case("letter-spacing: 0.1em !important", 1)]
    #[case("font-size: 20px; letter-spacing: 2px !important", 1)]
    #[case("letter-spacing: normal !important", 1)]
    #[case("letter-spacing: initial !important", 1)]
    #[case("letter-spacing: 0 !important", 1)]
    #[case("letter-spacing: 0.1em ! IMPORTANT", 1)]
    #[case("letter-spacing: 0.15em !important", 0)]
    #[case("font-size: 25px; letter-spacing: 3px !important", 0)]
    #[case(
        "letter-spacing: 0.1em !important; letter-spacing: 0.15em !important",
        0
    )]
    #[case("letter-spacing: 0.15em !important; letter-spacing: 0.1em", 0)]
    #[case("letter-spacing: 0.1em", 0)]
    #[case("letter-spacing: inherit !important", 0)]
    #[case("letter-spacing: unset !important", 0)]
    #[case("width: 60%", 0)]
    fn letter_spacing_24afc2(#[case] style: &str, #[case] expected: usize) {
        let html = format!(r#"<p style="{style}">{TEXT}</p>"#);
        assert_eq!(
            count(check_important_letter_spacing, &html),
            expected,
            "{style}"
        );
    }

    #[rstest]
    #[case(r#"<div style="letter-spacing: 0.1em !important;"></div>"#)]
    #[case(r#"<p hidden style="letter-spacing: 0.1em !important;">Text</p>"#)]
    #[case(r#"<div style="font-size: 16px; letter-spacing: 2px !important"><p style="font-size: 10px">Text</p></div>"#)]
    #[case(r#"<div style="letter-spacing: 0.1em !important"><p style="letter-spacing: 0.2em !important">Text</p></div>"#)]
    fn elements_without_their_own_visible_text_are_skipped(#[case] html: &str) {
        assert_eq!(count(check_important_letter_spacing, html), 0, "{html}");
    }

    #[rstest]
    #[case("word-spacing: 0.1em !important", 1)]
    #[case("font-size: 20px; word-spacing: 2px !important", 1)]
    #[case("word-spacing: normal !important", 1)]
    #[case("word-spacing: initial !important", 1)]
    #[case("word-spacing: 0.2em !important", 0)]
    #[case("font-size: 25px; word-spacing: 4px !important", 0)]
    #[case("word-spacing: 0.1em !important; word-spacing: 0.2em !important", 0)]
    #[case("word-spacing: 0.2em !important; word-spacing: 0.1em", 0)]
    #[case("word-spacing: 0.1em", 0)]
    #[case("word-spacing: inherit !important", 0)]
    fn word_spacing_9e45ec(#[case] style: &str, #[case] expected: usize) {
        let html = format!(r#"<p style="{style}">{TEXT}</p>"#);
        assert_eq!(
            count(check_important_word_spacing, &html),
            expected,
            "{style}"
        );
    }

    #[test]
    fn message_quotes_the_locked_value() {
        let page = page_from_html(r#"<p style="letter-spacing: 0.1em !important">Text</p>"#);
        let findings = check_important_letter_spacing(&page, &CheckOptions::default());
        assert_eq!(
            findings[0].message,
            r#"<p> locks letter-spacing at "0.1em !important" (0.10 times the font size, below 0.12)"#
        );
    }
}
