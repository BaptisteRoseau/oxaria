use super::{attributes, content_model, deprecated, native_attributes, roles, syntax};
use crate::rules::RuleCheck;

pub const RULE_IDS: &[&str] = &[
    "HTMLARIA002",
    "HTMLARIA003",
    "HTMLARIA004",
    "HTMLARIA007",
    "HTMLARIA009",
    "HTMLARIA010",
    "HTMLARIA011",
    "HTMLARIA013",
    "HTMLARIA014",
    "HTMLARIA015",
    "HTMLARIA016",
    "HTMLARIA017",
];

pub fn rule_checks() -> Vec<RuleCheck> {
    vec![
        roles::check_redundant_semantics,
        roles::check_generic_role,
        roles::check_abstract_role,
        attributes::check_disallowed_aria,
        attributes::check_hidden_focusable,
        attributes::check_hidden_twice,
        attributes::check_advised_against_aria,
        native_attributes::check_conflicting_aria,
        native_attributes::check_repeated_aria,
        deprecated::check_deprecated_aria,
        syntax::check_lowercase_tokens,
        content_model::check_disallowed_descendants,
    ]
}

#[cfg(test)]
mod tests {
    use crate::rules::Standard;
    use crate::rules::registry::tests::assert_every_rule_has_help;

    #[tokio::test]
    async fn every_rule_fires_with_help() {
        let page = r#"<header role="banner">H</header><article role="generic">A</article>
                   <div role="select">S</div><br aria-label="line break">
                   <button aria-hidden="true">Close</button>
                   <div hidden="until-found" aria-hidden="true">F</div>
                   <a href="/archive" aria-disabled="true">Archive</a>
                   <input type="checkbox" checked aria-checked="false">
                   <button disabled aria-disabled="true">Save</button>
                   <ul role="directory"><li>D</li></ul><div role="MAIN">M</div>
                   <div role="button"><button>B</button></div>"#;
        assert_every_rule_has_help(Standard::HtmlAria, &[page]).await;
    }
}
