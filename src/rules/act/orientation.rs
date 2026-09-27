//! ACT rule b33eff for orientation locked by CSS rotation. litehtml lays out
//! one viewport and never evaluates the other orientation, so this scans the
//! `<style>` text for rotations inside orientation media queries. Only a
//! rotation of a quarter turn by itself is caught: one relative to a
//! rotation declared outside the query (2.5deg, then 92.5deg) isn't.

use crate::page::RenderedPage;
use crate::rules::{CheckOptions, Finding};

/// How far from a quarter turn a rotation may be and still lock the page.
const ANGLE_TOLERANCE_DEGREES: f64 = 1.0;

struct Rotation {
    media: String,
    selector: String,
    declaration: String,
}

/// b33eff: an orientation media query must not rotate content by a quarter
/// turn to keep it in one orientation.
pub fn check_orientation_lock(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let css = strip_comments(&page.stylesheet_text());
    let mut rotations = Vec::new();
    collect_rotations(&css, None, &mut rotations);
    rotations
        .into_iter()
        .map(|rotation| {
            Finding::error(
                "b33eff",
                format!(
                    "\"{}\" in {} sets \"{}\", locking the page to one orientation",
                    rotation.selector, rotation.media, rotation.declaration
                ),
            )
            .help(
                "remove the rotation from the orientation media query and let the content \
                 reflow in both portrait and landscape",
            )
        })
        .collect()
}

fn collect_rotations(css: &str, media: Option<&str>, rotations: &mut Vec<Rotation>) {
    for (prelude, body) in blocks(css) {
        let nested_media = match is_orientation_query(prelude) {
            true => Some(prelude),
            false => media,
        };
        match (body.contains('{'), nested_media) {
            (true, _) => collect_rotations(body, nested_media, rotations),
            (false, Some(media)) => {
                rotations.extend(quarter_turns(body).map(|declaration| Rotation {
                    media: media.to_string(),
                    selector: prelude.to_string(),
                    declaration,
                }))
            }
            (false, None) => {}
        }
    }
}

/// Top-level `prelude { body }` pairs, bodies with their nested blocks.
fn blocks(css: &str) -> Vec<(&str, &str)> {
    let mut blocks = Vec::new();
    let mut rest = css;
    while let Some(open) = rest.find('{') {
        let Some(close) = matching_brace(&rest[open..]).map(|offset| open + offset) else {
            break;
        };
        let prelude = rest[..open].rsplit([';', '}']).next().unwrap_or_default();
        blocks.push((prelude.trim(), &rest[open + 1..close]));
        rest = &rest[close + 1..];
    }
    blocks
}

fn matching_brace(css: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, c) in css.char_indices() {
        match c {
            '{' => depth += 1,
            '}' if depth == 1 => return Some(index),
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn is_orientation_query(prelude: &str) -> bool {
    let compact: String = prelude
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    compact.starts_with("@media")
        && (compact.contains("orientation:portrait") || compact.contains("orientation:landscape"))
}

fn quarter_turns(body: &str) -> impl Iterator<Item = String> + '_ {
    body.split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .filter_map(|(name, value)| {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_ascii_lowercase();
            let angle = match name.as_str() {
                "transform" => transform_rotation(&value),
                "rotate" => rotate_property(&value),
                _ => None,
            }?;
            is_quarter_turn(angle).then(|| format!("{name}: {value}"))
        })
}

fn is_quarter_turn(degrees: f64) -> bool {
    let normalized = degrees.rem_euclid(360.0);
    [90.0, 270.0]
        .iter()
        .any(|quarter| (normalized - quarter).abs() <= ANGLE_TOLERANCE_DEGREES)
}

/// The `rotate` property turns around the Z axis unless given another axis.
fn rotate_property(value: &str) -> Option<f64> {
    let tokens: Vec<&str> = value.split_whitespace().collect();
    match tokens.as_slice() {
        [angle] | ["z", angle] => angle_degrees(angle),
        [x, y, z, angle] if is_z_axis(x, y, z) => {
            Some(angle_degrees(angle)? * z.parse::<f64>().ok()?.signum())
        }
        _ => None,
    }
}

/// The net Z rotation of the transform functions that can rotate.
fn transform_rotation(value: &str) -> Option<f64> {
    let angles: Vec<f64> = transform_functions(value)
        .filter_map(|(name, args)| function_rotation(name, &args))
        .collect();
    (!angles.is_empty()).then(|| angles.iter().sum())
}

fn transform_functions(value: &str) -> impl Iterator<Item = (&str, Vec<&str>)> {
    value.split(')').filter_map(|part| {
        let (name, args) = part.split_once('(')?;
        let args = args
            .split([',', ' '])
            .map(str::trim)
            .filter(|arg| !arg.is_empty())
            .collect();
        Some((name.trim(), args))
    })
}

fn function_rotation(name: &str, args: &[&str]) -> Option<f64> {
    let number = |index: usize| args.get(index)?.parse::<f64>().ok();
    match (name, args.len()) {
        ("rotate" | "rotatez", 1) => angle_degrees(args[0]),
        ("rotate3d", 4) if is_z_axis(args[0], args[1], args[2]) => {
            Some(angle_degrees(args[3])? * number(2)?.signum())
        }
        ("matrix", 6) | ("matrix3d", 16) => Some(number(1)?.atan2(number(0)?).to_degrees()),
        _ => None,
    }
}

fn is_z_axis(x: &str, y: &str, z: &str) -> bool {
    let value = |s: &str| s.parse::<f64>().ok();
    value(x) == Some(0.0) && value(y) == Some(0.0) && value(z).is_some_and(|z| z != 0.0)
}

fn angle_degrees(angle: &str) -> Option<f64> {
    let unit_start = angle
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(angle.len());
    let (number, unit) = angle.split_at(unit_start);
    let number: f64 = number.parse().ok()?;
    match unit {
        "deg" => Some(number),
        "rad" => Some(number.to_degrees()),
        "grad" => Some(number * 0.9),
        "turn" => Some(number * 360.0),
        "" if number == 0.0 => Some(0.0),
        _ => None,
    }
}

fn strip_comments(css: &str) -> String {
    let mut stripped = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        stripped.push_str(&rest[..start]);
        let comment = &rest[start + 2..];
        rest = comment.find("*/").map_or("", |end| &comment[end + 2..]);
    }
    stripped.push_str(rest);
    stripped
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn findings(css: &str) -> Vec<Finding> {
        let html = format!("<head><style>{css}</style></head><body>Page Content</body>");
        check_orientation_lock(&page_from_html(&html), &CheckOptions::default())
    }

    #[rstest]
    #[case(
        "@media (orientation: portrait) { html { transform: rotate(1.5708rad); width: 100vh; } }"
    )]
    #[case(
        "@media (orientation: landscape) { body { transform: matrix3d(0, -1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1); } }"
    )]
    #[case("@media (orientation: portrait) { html { rotate: 90deg; } }")]
    #[case("@media screen and (ORIENTATION:LANDSCAPE) { body { transform: rotateZ(-90deg) } }")]
    #[case(
        "@media (orientation: portrait) { body { transform: translateX(10px) rotate(0.25turn) } }"
    )]
    #[case(
        "@supports (rotate: 0deg) { @media (orientation: portrait) { main { rotate: z 270deg } } }"
    )]
    #[case("@media (orientation: portrait) { body { transform: rotate3d(0, 0, 1, 90deg) } }")]
    fn quarter_turns_in_orientation_queries_fail(#[case] css: &str) {
        assert_eq!(findings(css).len(), 1, "{css}");
    }

    #[rstest]
    #[case("@media (orientation: portrait) { html { transform: rotateZ(1turn); } }")]
    #[case(
        "@media (orientation: portrait) { html { transform: matrix(1, -1.22465e-15, 1.22465e-15, 1, 0, 0); } }"
    )]
    #[case("@media (orientation: portrait) { html { rotate: 0turn; } }")]
    #[case("html { font-size: 22px; } @media (min-width: 30em) { html { font-size: 100%; } }")]
    #[case("body { transform: rotate(90deg); }")]
    #[case("@media (orientation: portrait) { body { transform: translateX(100px); } }")]
    #[case("@media (orientation: portrait) { body { transform: rotate3d(1, 0, 0, 90deg) } }")]
    #[case("/* @media (orientation: portrait) { body { rotate: 90deg } } */")]
    fn other_rotations_pass(#[case] css: &str) {
        assert!(findings(css).is_empty(), "{css}");
    }

    #[test]
    fn message_names_the_rule_and_declaration() {
        let found = findings("@media (orientation: portrait) { html { rotate: 90deg; } }");
        assert_eq!(
            found[0].message,
            "\"html\" in @media (orientation: portrait) sets \"rotate: 90deg\", locking the page \
             to one orientation"
        );
    }
}
