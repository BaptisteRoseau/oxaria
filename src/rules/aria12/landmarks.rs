//! ARIA-LMK001: one `banner`, `main` and `contentinfo` per document.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{first_valid_role, is_hidden};
use crate::rules::{CheckOptions, Finding};

use super::tree::exposed_role;

const UNIQUE_LANDMARKS: &[(&str, &str)] = &[
    ("banner", "the site header"),
    ("main", "the page's main content"),
    ("contentinfo", "the site footer"),
];

/// Roles that start a document of their own, which may have its own
/// landmarks.
const NESTED_DOCUMENT_ROLES: &[&str] = &["document", "application"];

pub fn check_duplicate_landmark(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let landmarks: Vec<(ElementRef, &str, Option<ElementRef>)> = page
        .all()
        .filter(|el| !is_hidden(*el))
        .filter_map(|el| {
            let role = exposed_role(el).map(|role| role.name)?;
            let landmark = UNIQUE_LANDMARKS.iter().find(|(name, _)| *name == role)?.0;
            Some((el, landmark, nested_document(el)))
        })
        .collect();
    landmarks
        .iter()
        .enumerate()
        .filter_map(|(index, (el, landmark, document))| {
            let first = landmarks[..index]
                .iter()
                .find(|(_, other, other_document)| {
                    other == landmark && other_document == document
                })?;
            Some(duplicate_landmark_finding(*el, landmark, first.0))
        })
        .collect()
}

fn nested_document(el: ElementRef) -> Option<ElementRef> {
    el.ancestors().find(|ancestor| {
        first_valid_role(*ancestor).is_some_and(|role| NESTED_DOCUMENT_ROLES.contains(&role.name))
    })
}

fn duplicate_landmark_finding(el: ElementRef, landmark: &str, first: ElementRef) -> Finding {
    let purpose = UNIQUE_LANDMARKS
        .iter()
        .find(|(name, _)| *name == landmark)
        .map_or("", |(_, purpose)| purpose);
    Finding::error(
        "ARIA-LMK001",
        format!(
            "second \"{landmark}\" landmark in the document (the first is {})",
            first.selector()
        ),
    )
    .at(el)
    .help(format!(
        "keep one {landmark} landmark, for {purpose}; mark this one up as a region with a name, \
         or as complementary if it is related content"
    ))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(html: &str) -> Vec<Finding> {
        check_duplicate_landmark(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(r#"<div role="main">article</div><div role="main">related articles</div>"#)]
    #[case("<header>Site</header><main>x</main><header>Again</header>")]
    #[case(r#"<footer>a</footer><div role="contentinfo">b</div>"#)]
    fn duplicate_landmarks_are_flagged(#[case] html: &str) {
        assert_eq!(run(html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<div role="main">article</div><div role="complementary" aria-label="Related articles">x</div>"#)]
    #[case("<header>Site</header><main><article><header>Post</header></article></main>")]
    #[case("<main>a</main><main hidden>b</main>")]
    #[case(r#"<main>a</main><div role="application"><div role="main">b</div></div>"#)]
    fn single_landmarks_are_not_flagged(#[case] html: &str) {
        assert!(run(html).is_empty(), "{html}");
    }
}
