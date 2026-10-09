//! Minimal CSS text helpers for the rules that read raw stylesheets or `style` attributes.

pub fn strip_comments(css: &str) -> String {
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

/// Every `name: value` pair of a declaration block, both trimmed, in source order.
pub fn declarations(body: &str) -> impl DoubleEndedIterator<Item = (&str, &str)> {
    body.split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .map(|(name, value)| (name.trim(), value.trim()))
}

/// The value of the first declaration of `property` (ASCII case-insensitive).
pub fn declaration<'a>(body: &'a str, property: &str) -> Option<&'a str> {
    declarations(body)
        .find(|(name, _)| name.eq_ignore_ascii_case(property))
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_comments_removes_every_comment() {
        assert_eq!(
            strip_comments("a{/* } */x:y}/* b */c{}/* open"),
            "a{x:y}c{}"
        );
    }

    #[test]
    fn declarations_trims_names_and_values_and_skips_malformed_ones() {
        assert_eq!(
            declarations(" color : red ; junk;; margin:0 ").collect::<Vec<_>>(),
            [("color", "red"), ("margin", "0")]
        );
    }

    #[test]
    fn declaration_finds_the_first_match_case_insensitively() {
        let body = "OUTLINE: none !important; outline: 2px solid";
        assert_eq!(declaration(body, "outline"), Some("none !important"));
        assert_eq!(declaration(body, "border"), None);
    }
}
