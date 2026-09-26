//! WCAG 2.2.1 (Timing Adjustable) checks for `meta http-equiv="refresh"`.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::{CheckOptions, Finding};

/// F40/F41 treat a delay under one second as instant and one over 20 hours as never reached.
const SHORTEST_TIMED_DELAY: f64 = 1.0;
const LONGEST_TIMED_DELAY: f64 = 72_000.0;

struct Refresh {
    delay: f64,
    url: Option<String>,
}

/// F40 (timed redirect) / F41 (periodic reload): a `meta` refresh that fires after a delay
/// changes the page under users who may still be reading it, with no way to stop it.
pub fn check_timed_meta_refresh(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("meta")
        .filter(|meta| is_refresh(*meta))
        .filter_map(|meta| parse_refresh(meta.attr("content")?).map(|refresh| (meta, refresh)))
        .filter(|(_, refresh)| is_timed(refresh.delay))
        .map(|(meta, refresh)| refresh_finding(meta, &refresh))
        .collect()
}

fn is_refresh(meta: ElementRef) -> bool {
    meta.attr("http-equiv")
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("refresh"))
}

fn is_timed(delay: f64) -> bool {
    (SHORTEST_TIMED_DELAY..=LONGEST_TIMED_DELAY).contains(&delay)
}

fn parse_refresh(content: &str) -> Option<Refresh> {
    let content = content.trim_start();
    let number_end = content
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(content.len());
    let delay = content[..number_end].parse().ok()?;
    Some(Refresh {
        delay,
        url: refresh_url(&content[number_end..]),
    })
}

fn refresh_url(rest: &str) -> Option<String> {
    let rest = rest.trim_start_matches([' ', ';', ',']).trim();
    let url = match rest.get(..4) {
        Some(prefix) if prefix.eq_ignore_ascii_case("url=") => &rest[4..],
        _ => rest,
    };
    let url = url.trim().trim_matches(['\'', '"']);
    (!url.is_empty()).then(|| url.to_string())
}

fn refresh_finding(meta: ElementRef, refresh: &Refresh) -> Finding {
    let delay = refresh.delay;
    match &refresh.url {
        Some(url) => Finding::error(
            "F40",
            format!("<meta http-equiv=\"refresh\"> redirects to \"{url}\" after {delay}s"),
        )
        .at(meta)
        .help(
            "redirect on the server (an HTTP 301), or use a delay of 0; \
             otherwise let users follow a link when they are ready",
        ),
        None => Finding::error(
            "F41",
            format!("<meta http-equiv=\"refresh\"> reloads the page every {delay}s"),
        )
        .at(meta)
        .help("remove it, and let users reload or turn the automatic update off from the page"),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn findings(content: &str) -> Vec<Finding> {
        let html = format!(r#"<head><meta http-equiv="refresh" content="{content}"></head>"#);
        check_timed_meta_refresh(&page_from_html(&html), &CheckOptions::default())
    }

    #[test]
    fn timed_redirect_is_flagged() {
        let findings = findings("5; url=https://www.example.com/newpage");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F40");
        assert!(
            findings[0]
                .message
                .contains("\"https://www.example.com/newpage\" after 5s"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn periodic_reload_is_flagged() {
        let findings = findings("60");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F41");
    }

    #[rstest]
    #[case("0; url=https://www.example.com/newpage")]
    #[case("0.5")]
    #[case("72001")]
    #[case("not a number")]
    fn instant_or_unreachable_refresh_is_not_flagged(#[case] content: &str) {
        assert!(findings(content).is_empty(), "{content}");
    }

    #[rstest]
    #[case("3;URL='/next'", "/next")]
    #[case("3, url=/next", "/next")]
    #[case("3; /next", "/next")]
    fn redirect_url_forms_are_recognized(#[case] content: &str, #[case] url: &str) {
        let refresh = parse_refresh(content).unwrap();
        assert_eq!(refresh.url.as_deref(), Some(url));
    }

    #[test]
    fn other_meta_elements_are_ignored() {
        let p = page_from_html(r#"<head><meta name="viewport" content="5"></head>"#);
        assert!(check_timed_meta_refresh(&p, &CheckOptions::default()).is_empty());
    }
}
