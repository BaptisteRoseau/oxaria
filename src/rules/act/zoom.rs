//! ACT rule b4f0c3 for zoom restrictions in the viewport `meta` element.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::{CheckOptions, Finding};

/// `maximum-scale` below this stops users zooming to 200%.
const MINIMUM_MAXIMUM_SCALE: f64 = 2.0;

/// b4f0c3: a viewport `meta` must not disable zooming (`user-scalable`) or
/// cap it below 200% (`maximum-scale`).
pub fn check_viewport_zoom(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("meta")
        .filter(|meta| is_viewport(*meta))
        .filter_map(|meta| Some((meta, meta.attr("content")?)))
        .flat_map(|(meta, content)| {
            viewport_properties(content)
                .into_iter()
                .filter(|(key, value)| restricts_zoom(key, value))
                .map(move |(key, value)| zoom_finding(meta, &key, &value))
        })
        .collect()
}

fn is_viewport(meta: ElementRef) -> bool {
    meta.attr("name")
        .is_some_and(|name| name.trim().eq_ignore_ascii_case("viewport"))
}

/// CSS Device Adaptation's parsing: `key=value` pairs separated by commas,
/// semicolons or whitespace, with optional whitespace around `=`.
fn viewport_properties(content: &str) -> Vec<(String, String)> {
    let normalized = content
        .to_ascii_lowercase()
        .replace(['\t', '\n', '\r'], " ");
    let joined = normalized
        .split('=')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("=");
    joined
        .split([',', ';', ' '])
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn restricts_zoom(key: &str, value: &str) -> bool {
    match key {
        "user-scalable" => !allows_user_scaling(value),
        "maximum-scale" => !allows_maximum_scale(value),
        _ => false,
    }
}

/// Unknown values count as `no`, as browsers translate them to a fixed zoom.
fn allows_user_scaling(value: &str) -> bool {
    match value {
        "yes" | "device-width" | "device-height" => true,
        _ => leading_number(value).is_some_and(|number| number.abs() >= 1.0),
    }
}

/// Unknown values (`yes`, `invalid`) translate to a scale of 1 or less.
fn allows_maximum_scale(value: &str) -> bool {
    match value {
        "device-width" | "device-height" => true,
        _ => leading_number(value)
            .is_some_and(|number| number < 0.0 || number >= MINIMUM_MAXIMUM_SCALE),
    }
}

fn leading_number(value: &str) -> Option<f64> {
    let end = value
        .char_indices()
        .find(|(i, c)| !(c.is_ascii_digit() || *c == '.' || (*i == 0 && matches!(c, '-' | '+'))))
        .map_or(value.len(), |(i, _)| i);
    value[..end].parse().ok()
}

fn zoom_finding(meta: ElementRef, key: &str, value: &str) -> Finding {
    let help = match key {
        "user-scalable" => "remove user-scalable (or set it to yes) so users can pinch-zoom",
        _ => "remove maximum-scale, or set it to at least 2 so users can zoom to 200%",
    };
    let content = meta.attr("content").unwrap_or_default();
    Finding::error(
        "b4f0c3",
        format!("<meta name=\"viewport\" content=\"{content}\"> restricts zoom with {key}={value}"),
    )
    .at(meta)
    .help(help)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn count(content: &str) -> usize {
        let html = format!(r#"<head><meta name="viewport" content="{content}"></head>"#);
        check_viewport_zoom(&page_from_html(&html), &CheckOptions::default()).len()
    }

    #[rstest]
    #[case("user-scalable=no")]
    #[case("user-scalable=0.5")]
    #[case("user-scalable=invalid")]
    #[case("user-scalable=yes, initial-scale=0.8, maximum-scale=1.5")]
    #[case("maximum-scale=yes")]
    #[case("maximum-scale=invalid")]
    #[case("width=device-width, initial-scale=1, maximum-scale=1")]
    #[case("width=device-width; user-scalable = 0")]
    fn zoom_restrictions_fail(#[case] content: &str) {
        assert_eq!(count(content), 1, "{content}");
    }

    #[rstest]
    #[case("user-scalable=yes")]
    #[case("user-scalable=5")]
    #[case("maximum-scale=2.0")]
    #[case("maximum-scale=-1")]
    #[case("maximum-scale=device-width")]
    #[case("width=device-width, initial-scale=1")]
    #[case("")]
    #[case("USER-SCALABLE=YES")]
    fn allowed_zoom_passes(#[case] content: &str) {
        assert_eq!(count(content), 0, "{content}");
    }

    #[test]
    fn meta_without_content_or_other_names_is_ignored() {
        let page = page_from_html(
            r#"<head><meta name="viewport"><meta name="description" content="user-scalable=no"></head>"#,
        );
        assert!(check_viewport_zoom(&page, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn both_restrictions_are_reported() {
        assert_eq!(count("user-scalable=no, maximum-scale=1"), 2);
    }
}
