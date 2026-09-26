use crate::rules::RuleCheck;

pub fn rule_checks() -> Vec<RuleCheck> {
    vec![]
}

#[cfg(test)]
mod tests {
    use crate::rules::Standard;
    use crate::rules::registry::tests::assert_every_rule_has_help;

    #[tokio::test]
    async fn every_rule_fires_with_help() {
        assert_every_rule_has_help(Standard::HtmlAria, &[], 0).await;
    }
}
