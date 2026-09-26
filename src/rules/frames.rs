//! WCAG 4.1.2 check for `iframe` names.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

/// H64: an `iframe` needs a `title` (or ARIA name) so users can tell frames apart before
/// entering one.
pub fn check_untitled_iframe(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
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
    use super::*;
    use crate::page::testutil::page_from_html;

    fn findings(html: &str) -> Vec<Finding> {
        check_untitled_iframe(&page_from_html(html), &CheckOptions::default())
    }

    #[test]
    fn untitled_iframe_is_flagged() {
        let findings = findings(r#"<iframe src="banner-ad.html" name="ad-iframe"></iframe>"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H64");
    }

    #[test]
    fn empty_title_is_flagged() {
        assert_eq!(
            findings(r#"<iframe src="a.html" title=" "></iframe>"#).len(),
            1
        );
    }

    #[test]
    fn titled_iframe_is_not_flagged() {
        assert!(findings(r#"<iframe src="a.html" title="Advertisement"></iframe>"#).is_empty());
    }

    #[test]
    fn aria_named_iframe_is_not_flagged() {
        let html = r#"<h2 id="map">Store map</h2><iframe src="m.html" aria-labelledby="map"></iframe>
                      <iframe src="v.html" aria-label="Product video"></iframe>"#;
        assert!(findings(html).is_empty());
    }

    #[test]
    fn hidden_iframe_is_not_flagged() {
        let html = r#"<iframe src="t.html" hidden></iframe><iframe src="u.html" aria-hidden="true"></iframe>
                      <noscript><iframe src="https://www.googletagmanager.com/ns.html?id=GTM-X"></iframe></noscript>"#;
        assert!(findings(html).is_empty());
    }
}
