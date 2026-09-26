//! WCAG 1.1.1 (Non-text Content) checks for `img` elements.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

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
pub fn check_missing_alt(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("img")
        .filter(|img| img.attr("alt").is_none())
        .map(|img| Finding::error("F65", missing_alt_message(img)))
        .collect()
}

fn missing_alt_message(img: ElementRef) -> String {
    let src = img.attr("src").unwrap_or("(no src)");
    format!("<img src=\"{src}\"> has no alt attribute")
}

/// F30: an `alt` value that is a filename or a generic placeholder ("image", "spacer", …) is not
/// a real text alternative.
pub fn check_non_alternative_alt(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("img")
        .filter_map(non_alternative_finding)
        .collect()
}

fn non_alternative_finding(img: ElementRef) -> Option<Finding> {
    let alt = img.attr("alt")?.trim();
    is_non_alternative_alt(alt).then(|| Finding::error("F30", non_alternative_message(img, alt)))
}

fn is_non_alternative_alt(alt: &str) -> bool {
    !alt.is_empty() && (is_placeholder_word(alt) || looks_like_filename(alt))
}

fn is_placeholder_word(alt: &str) -> bool {
    let lower = alt.to_lowercase();
    PLACEHOLDER_ALT_VALUES.iter().any(|placeholder| {
        lower == *placeholder || lower.trim_end_matches(char::is_numeric) == *placeholder
    })
}

fn looks_like_filename(alt: &str) -> bool {
    let lower = alt.to_lowercase();
    let has_image_extension = [".jpg", ".jpeg", ".png", ".gif", ".svg", ".webp", ".bmp"]
        .iter()
        .any(|ext| lower.ends_with(ext));
    has_image_extension && !alt.contains(' ')
}

fn non_alternative_message(img: ElementRef, alt: &str) -> String {
    let src = img.attr("src").unwrap_or("(no src)");
    format!("<img src=\"{src}\"> has a non-alternative alt text: \"{alt}\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn options() -> CheckOptions {
        CheckOptions {
            contrast_threshold: 4.5,
            large_text_contrast_threshold: 3.0,
            target_size_threshold: 24.0,
        }
    }

    #[test]
    fn missing_alt_is_flagged() {
        let p = page_from_html(r#"<img src="photo.jpg">"#);
        let findings = check_missing_alt(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F65");
    }

    #[test]
    fn empty_alt_is_not_flagged_as_missing() {
        let p = page_from_html(r#"<img src="deco.png" alt="">"#);
        assert!(check_missing_alt(&p, &options()).is_empty());
    }

    #[test]
    fn descriptive_alt_is_not_flagged_as_missing() {
        let p = page_from_html(r#"<img src="photo.jpg" alt="A red bicycle">"#);
        assert!(check_missing_alt(&p, &options()).is_empty());
    }

    #[test]
    fn placeholder_word_alt_is_flagged() {
        let p = page_from_html(r#"<img src="a.jpg" alt="image">"#);
        let findings = check_non_alternative_alt(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F30");
    }

    #[test]
    fn numbered_placeholder_word_alt_is_flagged() {
        let p = page_from_html(r#"<img src="a.jpg" alt="picture1">"#);
        assert_eq!(check_non_alternative_alt(&p, &options()).len(), 1);
    }

    #[test]
    fn filename_alt_is_flagged() {
        let p = page_from_html(r#"<img src="a.jpg" alt="Oct.jpg">"#);
        assert_eq!(check_non_alternative_alt(&p, &options()).len(), 1);
    }

    #[test]
    fn descriptive_alt_is_not_flagged_as_non_alternative() {
        let p = page_from_html(r#"<img src="a.jpg" alt="Quarterly sales rose 12%">"#);
        assert!(check_non_alternative_alt(&p, &options()).is_empty());
    }

    #[test]
    fn empty_alt_is_not_flagged_as_non_alternative() {
        let p = page_from_html(r#"<img src="a.jpg" alt="">"#);
        assert!(check_non_alternative_alt(&p, &options()).is_empty());
    }
}
