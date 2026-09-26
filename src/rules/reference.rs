//! Links from a rule ID to its W3C documentation page.

const TECHNIQUES_URL: &str = "https://www.w3.org/WAI/WCAG22/Techniques";
const UNDERSTANDING_URL: &str = "https://www.w3.org/WAI/WCAG22/Understanding";

/// `None` for IDs that aren't WCAG rules (`FETCH`, `HTTP`, `SCAN`, ...).
pub fn reference_url(rule_id: &str) -> Option<String> {
    match rule_id {
        "TGT001" => Some(format!("{UNDERSTANDING_URL}/target-size-minimum")),
        _ => technique_url(rule_id),
    }
}

fn technique_url(rule_id: &str) -> Option<String> {
    let number_start = rule_id.find(|c: char| c.is_ascii_digit())?;
    let (prefix, number) = rule_id.split_at(number_start);
    if !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let directory = match prefix {
        "H" => "html",
        "G" => "general",
        "F" => "failures",
        "C" => "css",
        "ARIA" => "aria",
        _ => return None,
    };
    Some(format!("{TECHNIQUES_URL}/{directory}/{rule_id}"))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("H44", "https://www.w3.org/WAI/WCAG22/Techniques/html/H44")]
    #[case("G18", "https://www.w3.org/WAI/WCAG22/Techniques/general/G18")]
    #[case("F65", "https://www.w3.org/WAI/WCAG22/Techniques/failures/F65")]
    #[case("ARIA16", "https://www.w3.org/WAI/WCAG22/Techniques/aria/ARIA16")]
    #[case(
        "TGT001",
        "https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum"
    )]
    fn rule_ids_link_to_their_page(#[case] rule_id: &str, #[case] expected: &str) {
        assert_eq!(reference_url(rule_id).as_deref(), Some(expected));
    }

    #[rstest]
    #[case("FETCH")]
    #[case("HTTP")]
    #[case("SCAN")]
    #[case("INPUT")]
    #[case("RENDER")]
    fn non_wcag_ids_have_no_link(#[case] rule_id: &str) {
        assert_eq!(reference_url(rule_id), None);
    }
}
