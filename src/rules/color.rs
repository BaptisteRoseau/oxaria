//! Color and contrast math shared by WCAG's G18/G145 and ACT's afw4f7 contrast rules.

use crate::page::{ElementRef, Rgb};
use crate::rules::CheckOptions;

pub(crate) const DEFAULT_CANVAS_BACKGROUND: Rgb = (255, 255, 255);
const BLACK: Rgb = (0, 0, 0);
const WHITE: Rgb = (255, 255, 255);

/// The configured threshold for `el`'s text: the large-text one when it is large scale.
pub(crate) fn required_threshold(el: ElementRef, options: &CheckOptions) -> f64 {
    match is_large_text(el) {
        true => options.large_text_contrast_threshold,
        false => options.contrast_threshold,
    }
}

/// WCAG defines "large scale" text as at least 18pt (24px), or at least 14pt (18.66px) bold.
pub(crate) fn is_large_text(el: ElementRef) -> bool {
    let size = el.font_size_px();
    size >= 24.0 || (el.font_weight() >= 700 && size >= 18.66)
}

/// How to make `foreground` reach `threshold` on `background`: the nearest passing text color,
/// or a new background when no text color passes.
pub(crate) fn fix_suggestion(foreground: Rgb, background: Rgb, threshold: f64) -> String {
    match passing_color(foreground, background, threshold) {
        Some((verb, color)) => format!(
            "{verb} the text to {} ({:.2}:1 on {})",
            hex(color),
            color_contrast(color, background),
            hex(background)
        ),
        None => format!(
            "change the background: no text color reaches {threshold:.2}:1 on {}",
            hex(background)
        ),
    }
}

/// The color closest to `foreground`, on the way to black or white (whichever contrasts more
/// with `background`), that reaches `threshold`.
fn passing_color(foreground: Rgb, background: Rgb, threshold: f64) -> Option<(&'static str, Rgb)> {
    let (verb, extreme) =
        match color_contrast(BLACK, background) >= color_contrast(WHITE, background) {
            true => ("darken", BLACK),
            false => ("lighten", WHITE),
        };
    let passes =
        |amount: f64| color_contrast(mix(foreground, extreme, amount), background) >= threshold;
    if !passes(1.0) {
        return None;
    }
    let (mut failing, mut passing) = (0.0, 1.0);
    for _ in 0..32 {
        let middle = (failing + passing) / 2.0;
        match passes(middle) {
            true => passing = middle,
            false => failing = middle,
        }
    }
    Some((verb, mix(foreground, extreme, passing)))
}

fn mix((r1, g1, b1): Rgb, (r2, g2, b2): Rgb, amount: f64) -> Rgb {
    let channel =
        |from: u8, to: u8| (from as f64 + (to as f64 - from as f64) * amount).round() as u8;
    (channel(r1, r2), channel(g1, g2), channel(b1, b2))
}

pub(crate) fn hex((r, g, b): Rgb) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub(crate) fn color_contrast(first: Rgb, second: Rgb) -> f64 {
    contrast_ratio(relative_luminance(first), relative_luminance(second))
}

fn relative_luminance((r, g, b): Rgb) -> f64 {
    let channel = |value: u8| -> f64 {
        let normalized = value as f64 / 255.0;
        match normalized <= 0.04045 {
            true => normalized / 12.92,
            false => ((normalized + 0.055) / 1.055).powf(2.4),
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let (lighter, darker) = match l1 > l2 {
        true => (l1, l2),
        false => (l2, l1),
    };
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case((0, 0, 0), 0.0)]
    #[case((255, 255, 255), 1.0)]
    fn relative_luminance_of_black_and_white(#[case] color: Rgb, #[case] expected: f64) {
        assert!((relative_luminance(color) - expected).abs() < 1e-6);
    }

    #[test]
    fn contrast_ratio_black_on_white_is_twenty_one() {
        let ratio = contrast_ratio(
            relative_luminance((0, 0, 0)),
            relative_luminance((255, 255, 255)),
        );
        assert!((ratio - 21.0).abs() < 0.01);
    }
}
