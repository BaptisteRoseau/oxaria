//! Small helpers shared by the WAI-ARIA 1.2 rules.

use crate::page::ElementRef;
use crate::rules::aria_spec::{Role, first_valid_role, is_aria_true};

/// `role "button"`, noting when the role is implicit: authors don't
/// always know `div` is `generic` or `input` is `textbox`.
pub fn role_label(el: ElementRef, role: &Role) -> String {
    match first_valid_role(el) {
        Some(explicit) if explicit.name == role.name => format!("role \"{}\"", role.name),
        _ => format!("role \"{}\" (implicit on <{}>)", role.name, el.tag()),
    }
}

/// A custom element may get its role from script (`ElementInternals`),
/// which a static render can't see, so its implicit `generic` role isn't
/// trusted.
pub fn is_unroled_custom_element(el: ElementRef) -> bool {
    el.tag().contains('-') && first_valid_role(el).is_none()
}

pub fn is_aria_disabled(el: ElementRef) -> bool {
    el.self_and_ancestors()
        .any(|el| is_aria_true(el, "aria-disabled"))
}

/// "a, b or c".
pub fn or_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [only] => (*only).to_string(),
        [init @ .., last] => format!("{} or {last}", init.join(", ")),
    }
}

/// The candidate closest to `word`, when it's close enough to be a typo.
pub fn closest<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (word.len() / 4).max(2);
    candidates
        .into_iter()
        .map(|candidate| (edit_distance(word, candidate), candidate))
        .filter(|(distance, _)| *distance <= limit)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != *cb);
            current.push(substitution.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_close_spellings_only() {
        let names = ["aria-labelledby", "aria-label"];
        assert_eq!(closest("aria-labeledby", names), Some("aria-labelledby"));
        assert_eq!(closest("aria-foo", ["aria-labelledby"]), None);
    }

    #[test]
    fn lists_with_or() {
        assert_eq!(or_list(&["a"]), "a");
        assert_eq!(or_list(&["a", "b", "c"]), "a, b or c");
    }
}
