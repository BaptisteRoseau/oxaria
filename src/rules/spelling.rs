//! "Did you mean" suggestions for misspelled names and values.

/// The first candidate nearest to `word`, if it's within `limit` edits.
pub fn closest_within<'a>(
    word: &str,
    candidates: impl IntoIterator<Item = &'a str>,
    limit: usize,
) -> Option<&'a str> {
    candidates
        .into_iter()
        .map(|candidate| (edit_distance(word, candidate), candidate))
        .filter(|(distance, _)| *distance <= limit)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

/// Levenshtein distance, in chars.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, a_char) in a.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, b_char) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(a_char != *b_char);
            current.push(substitution.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("lnik", "link", 2)]
    #[case("aria-labelled", "aria-labelledby", 2)]
    #[case("", "abc", 3)]
    #[case("same", "same", 0)]
    fn edit_distances(#[case] a: &str, #[case] b: &str, #[case] expected: usize) {
        assert_eq!(edit_distance(a, b), expected);
    }

    #[test]
    fn ties_go_to_the_first_candidate() {
        assert_eq!(closest_within("ab", ["ax", "ay"], 2), Some("ax"));
        assert_eq!(closest_within("ab", ["xy"], 1), None);
    }
}
