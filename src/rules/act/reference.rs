const RULES_URL: &str = "https://www.w3.org/WAI/standards-guidelines/act/rules";

pub fn reference_url(rule_id: &str) -> String {
    format!("{RULES_URL}/{rule_id}/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_link_to_their_page_by_id() {
        assert_eq!(
            reference_url("2779a5"),
            "https://www.w3.org/WAI/standards-guidelines/act/rules/2779a5/"
        );
    }
}
