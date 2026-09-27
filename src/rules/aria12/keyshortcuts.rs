//! ARIA-ATTR007: the syntax of `aria-keyshortcuts`.

use crate::page::RenderedPage;
use crate::rules::{CheckOptions, Finding};

use super::support::quoted;

const MODIFIERS: &[&str] = &["Alt", "Control", "Shift", "Meta", "AltGraph"];

/// Common names for modifiers that aren't their UI Events key values.
const MODIFIER_ALIASES: &[(&str, &str)] = &[
    ("ctrl", "Control"),
    ("cmd", "Meta"),
    ("command", "Meta"),
    ("win", "Meta"),
    ("windows", "Meta"),
    ("super", "Meta"),
    ("option", "Alt"),
    ("opt", "Alt"),
    ("altgr", "AltGraph"),
];

/// ARIA-ATTR007: each shortcut is modifiers first, then exactly one other
/// key, joined by `+`; shortcuts are separated by spaces.
pub fn check_keyshortcuts_syntax(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            let value = el.attr("aria-keyshortcuts").unwrap_or_default();
            value
                .split_ascii_whitespace()
                .filter(|shortcut| !is_valid(shortcut))
                .map(move |shortcut| {
                    Finding::error(
                        "ARIA-ATTR007",
                        format!(
                            "\"{shortcut}\" in {} is not a valid key shortcut",
                            quoted("aria-keyshortcuts", value)
                        ),
                    )
                    .at(el)
                    .help(shortcut_help(shortcut))
                })
        })
        .collect()
}

/// The keys of a shortcut, with `+` itself as a key (`Shift++`).
fn keys(shortcut: &str) -> Vec<&str> {
    match shortcut.strip_suffix("++") {
        Some(modifiers) => modifiers.split('+').chain(["+"]).collect(),
        None if shortcut == "+" => vec!["+"],
        None => shortcut.split('+').collect(),
    }
}

fn is_valid(shortcut: &str) -> bool {
    let keys = keys(shortcut);
    let Some((key, modifiers)) = keys.split_last() else {
        return false;
    };
    !key.is_empty() && !is_modifier(key) && modifiers.iter().all(|m| is_modifier(m))
}

fn is_modifier(key: &str) -> bool {
    MODIFIERS
        .iter()
        .any(|modifier| modifier.eq_ignore_ascii_case(key))
}

fn shortcut_help(shortcut: &str) -> String {
    let syntax = "modifiers (Alt, Control, Shift, Meta, AltGraph) first, then exactly one other \
                  key, joined by \"+\"";
    match corrected(shortcut) {
        Some(correction) => format!("write it as \"{correction}\": {syntax}"),
        None => format!("write each shortcut as {syntax}, e.g. \"Control+Shift+S\""),
    }
}

/// The shortcut with aliases replaced and modifiers moved first, when it
/// has exactly one non-modifier key.
fn corrected(shortcut: &str) -> Option<String> {
    let keys: Vec<&str> = keys(shortcut).into_iter().map(canonical).collect();
    let (modifiers, others): (Vec<&str>, Vec<&str>) = keys.iter().partition(|key| is_modifier(key));
    match others.as_slice() {
        [key] if !key.is_empty() => Some(
            modifiers
                .into_iter()
                .chain([*key])
                .collect::<Vec<_>>()
                .join("+"),
        ),
        _ => None,
    }
}

fn canonical(key: &str) -> &str {
    MODIFIER_ALIASES
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(key))
        .map_or(key, |(_, modifier)| modifier)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(html: &str) -> Vec<Finding> {
        check_keyshortcuts_syntax(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case("Ctrl+S", "Control+S")]
    #[case("T+Shift+Alt", "Shift+Alt+T")]
    #[case("Cmd+Option+P", "Meta+Alt+P")]
    fn invalid_shortcuts_get_a_correction(#[case] shortcut: &str, #[case] correction: &str) {
        let findings = run(&format!(
            r#"<button aria-keyshortcuts="{shortcut}">x</button>"#
        ));
        assert_eq!(findings.len(), 1);
        assert!(
            findings[0]
                .help
                .as_deref()
                .unwrap()
                .starts_with(&format!("write it as \"{correction}\"")),
            "{findings:?}"
        );
    }

    #[rstest]
    #[case("Shift")]
    #[case("Control+")]
    #[case("A+B")]
    fn shortcuts_without_one_key_are_flagged(#[case] shortcut: &str) {
        let findings = run(&format!(
            r#"<button aria-keyshortcuts="{shortcut}">x</button>"#
        ));
        assert_eq!(findings.len(), 1, "{shortcut}");
    }

    #[rstest]
    #[case("Control+S")]
    #[case("Alt+Shift+T")]
    #[case("Alt+Shift+P Control+F")]
    #[case("Shift+Space")]
    #[case("A")]
    #[case("Shift++")]
    #[case("control+s")]
    #[case("")]
    fn valid_shortcuts_are_not_flagged(#[case] shortcut: &str) {
        let findings = run(&format!(
            r#"<button aria-keyshortcuts="{shortcut}">x</button>"#
        ));
        assert!(findings.is_empty(), "{shortcut}: {findings:?}");
    }

    #[test]
    fn each_invalid_shortcut_is_reported() {
        assert_eq!(
            run(r#"<button aria-keyshortcuts="Ctrl+S Alt+F Win+E">x</button>"#).len(),
            2
        );
    }
}
