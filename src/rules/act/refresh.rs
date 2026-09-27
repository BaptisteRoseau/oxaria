//! ACT rules for `meta http-equiv="refresh"` delays. Only the first
//! `meta` refresh whose `content` is valid counts, as HTML only acts on it.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::{CheckOptions, Finding};

/// A delay above 20 hours is considered never reached by bc659a.
const LONGEST_TIMED_DELAY: u64 = 72_000;

/// bc659a: the first valid meta refresh must be instant or wait more than
/// 20 hours.
pub fn check_refresh_delay(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    first_valid_refresh(page)
        .filter(|(_, delay)| (1..=LONGEST_TIMED_DELAY).contains(delay))
        .map(|(meta, delay)| {
            refresh_finding("bc659a", meta, delay).help(
                "redirect on the server (an HTTP 301) or with a delay of 0, and let users \
                 reload or update the page themselves instead of refreshing it on a timer",
            )
        })
        .into_iter()
        .collect()
}

/// bisz58: the first valid meta refresh must be instant, however long its
/// delay.
pub fn check_refresh_instant(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    first_valid_refresh(page)
        .filter(|(_, delay)| *delay > 0)
        .map(|(meta, delay)| {
            refresh_finding("bisz58", meta, delay).help(
                "use a delay of 0 (or an HTTP redirect); at Level AAA even a refresh after \
                 20 hours interrupts the user",
            )
        })
        .into_iter()
        .collect()
}

fn refresh_finding(rule_id: &'static str, meta: ElementRef, delay: u64) -> Finding {
    let content = meta.attr("content").unwrap_or_default();
    Finding::error(
        rule_id,
        format!("<meta http-equiv=\"refresh\" content=\"{content}\"> refreshes after {delay}s"),
    )
    .at(meta)
}

fn first_valid_refresh(page: &RenderedPage) -> Option<(ElementRef<'_>, u64)> {
    page.by_tag("meta")
        .filter(|meta| is_refresh(*meta))
        .find_map(|meta| Some((meta, refresh_delay(meta.attr("content")?)?)))
}

fn is_refresh(meta: ElementRef) -> bool {
    meta.attr("http-equiv")
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("refresh"))
}

/// HTML's shared declarative refresh steps, up to the delay: `None` when
/// the content is invalid and the browser ignores it (`"0: url"`, `"+5"`,
/// `""`).
fn refresh_delay(content: &str) -> Option<u64> {
    let content = content.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let digits_end = content
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(content.len());
    let (digits, rest) = content.split_at(digits_end);
    let delay = match digits {
        "" if rest.starts_with('.') => 0,
        "" => return None,
        digits => digits.parse().unwrap_or(u64::MAX),
    };
    let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
    let is_valid_separator = rest
        .chars()
        .next()
        .is_none_or(|c| c == ';' || c == ',' || c.is_ascii_whitespace());
    is_valid_separator.then_some(delay)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;
    use crate::rules::RuleCheck;

    fn count(check: RuleCheck, head: &str) -> usize {
        let html = format!("<head>{head}</head><body><p>x</p></body>");
        check(&page_from_html(&html), &CheckOptions::default()).len()
    }

    const INSTANT_REDIRECT: &str =
        r#"<meta http-equiv="refresh" content="0; URL='https://github.com'">"#;
    const INSTANT_THEN_TIMED: &str = r#"<meta http-equiv="refresh" content="0; https://w3.org"><meta http-equiv="refresh" content="5; https://w3.org">"#;
    const AFTER_20_HOURS: &str = r#"<meta http-equiv="refresh" content="72001; https://w3.org">"#;
    const RELOAD_30S: &str = r#"<meta http-equiv="refresh" content="30">"#;
    const REDIRECT_30S: &str = r#"<meta http-equiv="refresh" content="30; URL='https://w3.org'">"#;
    const INVALID_THEN_TIMED: &str = r#"<meta http-equiv="refresh" content="0: https://w3.org"><meta http-equiv="refresh" content="5; https://w3.org">"#;
    const EXACTLY_20_HOURS: &str = r#"<meta http-equiv="refresh" content="72000; https://w3.org">"#;

    #[rstest]
    #[case(RELOAD_30S, 1)]
    #[case(REDIRECT_30S, 1)]
    #[case(INVALID_THEN_TIMED, 1)]
    #[case(EXACTLY_20_HOURS, 1)]
    #[case(INSTANT_REDIRECT, 0)]
    #[case(INSTANT_THEN_TIMED, 0)]
    #[case(AFTER_20_HOURS, 0)]
    #[case(r#"<meta http-equiv="refresh">"#, 0)]
    #[case(r#"<meta content="30">"#, 0)]
    #[case(r#"<meta http-equiv="refresh" content="0: https://w3.org">"#, 0)]
    fn delay_bc659a(#[case] head: &str, #[case] expected: usize) {
        assert_eq!(count(check_refresh_delay, head), expected, "{head}");
    }

    #[rstest]
    #[case(RELOAD_30S, 1)]
    #[case(AFTER_20_HOURS, 1)]
    #[case(r#"<meta http-equiv="refresh" content="0: x"><meta http-equiv="refresh" content="72001; x">"#, 1)]
    #[case(INSTANT_REDIRECT, 0)]
    #[case(
        r#"<meta http-equiv="refresh" content="0; x"><meta http-equiv="refresh" content="30; x">"#,
        0
    )]
    #[case(r#"<meta http-equiv="refresh" content="; 72001">"#, 0)]
    fn instant_bisz58(#[case] head: &str, #[case] expected: usize) {
        assert_eq!(count(check_refresh_instant, head), expected, "{head}");
    }

    #[rstest]
    #[case("30", Some(30))]
    #[case("  5; url=/x", Some(5))]
    #[case("5,/x", Some(5))]
    #[case("3.9 /x", Some(3))]
    #[case(".5", Some(0))]
    #[case("0: https://w3.org", None)]
    #[case("-00.12 foo", None)]
    #[case("; 30", None)]
    #[case("", None)]
    #[case("+5; https://w3.org", None)]
    #[case("foo; URL='https://w3.org'", None)]
    fn parses_delays(#[case] content: &str, #[case] expected: Option<u64>) {
        assert_eq!(refresh_delay(content), expected, "{content}");
    }
}
