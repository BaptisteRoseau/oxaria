//! WCAG 2.4.2 (Page Titled) checks for the document's `title` element.

use crate::page::RenderedPage;

use crate::rules::Finding;

/// Authoring-tool defaults and filler text listed by F25, lowercased.
const PLACEHOLDER_TITLES: &[&str] = &[
    "enter the title of your html document here",
    "untitled document",
    "untitled page",
    "untitled",
    "no title",
    "new page",
    "new document",
    "title",
    "page title",
];

const PAGE_EXTENSIONS: &[&str] = &[".html", ".htm", ".php", ".asp", ".aspx", ".jsp"];

/// H25: the document needs a non-empty `title` element in its `head`.
pub fn check_missing_title(page: &RenderedPage) -> Vec<Finding> {
    match document_title(page).is_some_and(|title| !title.is_empty()) {
        true => Vec::new(),
        false => vec![
            Finding::error("H25", "page has no non-empty <title> element").help(
                "add a <title> to the <head> saying what the page is about, \
                 e.g. <title>Order summary - Acme</title>",
            ),
        ],
    }
}

/// F25: a title that is an authoring-tool default, filler text, or a bare filename does not
/// identify the page.
pub fn check_placeholder_title(page: &RenderedPage) -> Vec<Finding> {
    document_title(page)
        .filter(|title| is_placeholder_title(title))
        .map(|title| {
            Finding::error(
                "F25",
                format!("page title \"{title}\" does not identify the page"),
            )
            .help(
                "replace it with a title describing this page's content or purpose, \
                 e.g. <title>Order summary - Acme</title>",
            )
        })
        .into_iter()
        .collect()
}

/// An `svg` can hold its own `title` element, so only the one in `head` counts.
fn document_title(page: &RenderedPage) -> Option<String> {
    page.by_tag("title")
        .find(|title| title.has_ancestor("head"))
        .map(|title| title.text())
}

fn is_placeholder_title(title: &str) -> bool {
    let lower = title.trim().to_lowercase();
    let without_number = lower.trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ');
    PLACEHOLDER_TITLES.contains(&without_number) || looks_like_page_filename(&lower)
}

fn looks_like_page_filename(title: &str) -> bool {
    !title.contains(' ') && PAGE_EXTENSIONS.iter().any(|ext| title.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::{rule_ids, run};

    #[rstest]
    #[case("<html><head></head><body><p>x</p></body></html>", &["H25"])]
    #[case("<html><head><title>  </title></head></html>", &["H25"])]
    // An svg title doesn't count as the page title.
    #[case("<html><body><svg><title>Icon</title></svg></body></html>", &["H25"])]
    #[case("<html><head><title>Contact us</title></head></html>", &[])]
    fn missing_title(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_missing_title, html), expected, "{html}");
    }

    #[rstest]
    #[case("Untitled Document")]
    #[case("New Page 1")]
    #[case("Enter the title of your HTML document here")]
    #[case("spk12.html")]
    fn placeholder_title_is_flagged(#[case] title: &str) {
        let html = format!("<html><head><title>{title}</title></head></html>");
        let findings = run(check_placeholder_title, &html);
        assert_eq!(findings.len(), 1, "{title}");
        assert_eq!(findings[0].rule_id, "F25");
    }

    #[rstest]
    #[case("The World Wide Web Consortium")]
    #[case("Untitled Goose Game")]
    #[case("HTML 5.2 changes")]
    fn descriptive_title_is_not_flagged(#[case] title: &str) {
        let html = format!("<html><head><title>{title}</title></head></html>");
        assert!(run(check_placeholder_title, &html).is_empty());
    }
}
