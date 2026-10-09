//! WCAG 1.1.1 (Non-text Content) checks for `img` elements.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

const PLACEHOLDER_ALT_VALUES: &[&str] = &[
    "image",
    "img",
    "picture",
    "photo",
    "graphic",
    "spacer",
    "placeholder",
];

/// F65: every `img` must carry an `alt` attribute, even an empty one.
pub fn check_missing_alt(page: &RenderedPage) -> Vec<Finding> {
    page.by_tag("img")
        .filter(|img| img.attr("alt").is_none())
        .map(|img| {
            Finding::error_at(img, "F65", missing_alt_message(img)).help(missing_alt_help(img))
        })
        .collect()
}

fn missing_alt_message(img: ElementRef) -> String {
    let src = img.attr("src").unwrap_or("(no src)");
    format!("<img src=\"{src}\"> has no alt attribute")
}

fn missing_alt_help(img: ElementRef) -> String {
    let src = img.attr("src").unwrap_or("...");
    format!(
        "describe what the image shows, e.g. <img src=\"{src}\" alt=\"...\">, \
         or use alt=\"\" if it is purely decorative"
    )
}

/// F30: an `alt` value that is a filename or a generic placeholder ("image", "spacer", …) is not
/// a real text alternative.
pub fn check_non_alternative_alt(page: &RenderedPage) -> Vec<Finding> {
    page.by_tag("img")
        .filter_map(non_alternative_finding)
        .collect()
}

fn non_alternative_finding(img: ElementRef) -> Option<Finding> {
    let alt = img.attr("alt")?.trim();
    is_non_alternative_alt(alt).then(|| {
        Finding::error_at(img, "F30", non_alternative_message(img, alt))
            .help(non_alternative_help(alt))
    })
}

fn is_non_alternative_alt(alt: &str) -> bool {
    !alt.is_empty() && (is_placeholder_word(alt) || looks_like_image_filename(alt))
}

fn is_placeholder_word(alt: &str) -> bool {
    let lower = alt.to_lowercase();
    PLACEHOLDER_ALT_VALUES.iter().any(|placeholder| {
        lower == *placeholder || lower.trim_end_matches(char::is_numeric) == *placeholder
    })
}

fn looks_like_image_filename(alt: &str) -> bool {
    let lower = alt.to_lowercase();
    let has_image_extension = [".jpg", ".jpeg", ".png", ".gif", ".svg", ".webp", ".bmp"]
        .iter()
        .any(|ext| lower.ends_with(ext));
    has_image_extension && !alt.contains(' ')
}

fn non_alternative_help(alt: &str) -> String {
    format!(
        "replace \"{alt}\" with the information the image conveys (for a chart, its key \
         figures), or use alt=\"\" if it is purely decorative"
    )
}

fn non_alternative_message(img: ElementRef, alt: &str) -> String {
    let src = img.attr("src").unwrap_or("(no src)");
    format!("<img src=\"{src}\"> has a non-alternative alt text: \"{alt}\"")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::{help, rule_ids};

    #[test]
    fn missing_alt_help_reuses_the_src() {
        let help = help(check_missing_alt, r#"<img src="photo.jpg">"#);
        assert!(
            help.contains(r#"<img src="photo.jpg" alt="...">"#),
            "{help}"
        );
    }

    #[test]
    fn non_alternative_help_quotes_the_alt() {
        let html = r#"<img src="a.jpg" alt="chart.jpg">"#;
        let help = help(check_non_alternative_alt, html);
        assert!(help.starts_with(r#"replace "chart.jpg" with"#), "{help}");
    }

    #[rstest]
    #[case(r#"<img src="photo.jpg">"#, &["F65"])]
    #[case(r#"<img src="deco.png" alt="">"#, &[])]
    #[case(r#"<img src="photo.jpg" alt="A red bicycle">"#, &[])]
    fn missing_alt(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(rule_ids(check_missing_alt, html), expected, "{html}");
    }

    #[rstest]
    #[case(r#"<img src="a.jpg" alt="image">"#, &["F30"])]
    #[case(r#"<img src="a.jpg" alt="picture1">"#, &["F30"])]
    #[case(r#"<img src="a.jpg" alt="Oct.jpg">"#, &["F30"])]
    #[case(r#"<img src="a.jpg" alt="Quarterly sales rose 12%">"#, &[])]
    #[case(r#"<img src="a.jpg" alt="">"#, &[])]
    fn non_alternative_alt(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(
            rule_ids(check_non_alternative_alt, html),
            expected,
            "{html}"
        );
    }
}
