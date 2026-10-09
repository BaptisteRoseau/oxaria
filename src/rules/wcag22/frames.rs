//! WCAG 4.1.2 check for `iframe` names.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

/// H64: an `iframe` needs a `title` (or ARIA name) so users can tell frames apart before
/// entering one.
pub fn check_untitled_iframe(page: &RenderedPage) -> Vec<Finding> {
    page.by_tag("iframe")
        .filter(|iframe| !is_hidden(*iframe))
        .filter(|iframe| !has_name(page, *iframe))
        .map(|iframe| {
            let src = iframe.attr("src").unwrap_or("(no src)");
            Finding::error("H64", format!("<iframe src=\"{src}\"> has no title"))
                .at(iframe)
                .help(
                    "add a title describing the frame's content, \
                     e.g. <iframe src=\"...\" title=\"Advertisement\">",
                )
        })
        .collect()
}

/// Browsers with scripting enabled never render `noscript` content, which is where tag managers
/// put their fallback tracking iframe.
fn is_hidden(iframe: ElementRef) -> bool {
    iframe.has_attr("hidden")
        || iframe.attr("aria-hidden") == Some("true")
        || iframe.ancestors().any(|el| el.tag() == "noscript")
}

fn has_name(page: &RenderedPage, iframe: ElementRef) -> bool {
    let named_by = |attr| iframe.attr(attr).is_some_and(|v| !v.trim().is_empty());
    named_by("title")
        || named_by("aria-label")
        || iframe
            .attr("aria-labelledby")
            .is_some_and(|ids| !page.ids_text(ids).is_empty())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::rule_ids;

    #[rstest]
    #[case(r#"<iframe src="banner-ad.html" name="ad-iframe"></iframe>"#, &["H64"])]
    #[case(r#"<iframe src="a.html" title=" "></iframe>"#, &["H64"])]
    #[case(r#"<iframe src="a.html" title="Advertisement"></iframe>"#, &[])]
    #[case(
        r#"<h2 id="map">Store map</h2><iframe src="m.html" aria-labelledby="map"></iframe>
           <iframe src="v.html" aria-label="Product video"></iframe>"#,
        &[]
    )]
    #[case(
        r#"<iframe src="t.html" hidden></iframe><iframe src="u.html" aria-hidden="true"></iframe>
           <noscript><iframe src="https://www.googletagmanager.com/ns.html?id=GTM-X"></iframe></noscript>"#,
        &[]
    )]
    fn untitled_iframe(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_untitled_iframe, html), expected, "{html}");
    }
}
