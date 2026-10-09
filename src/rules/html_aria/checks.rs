use super::{
    attributes, content_model, deprecated, native_attributes, reference::reference_url, roles,
    syntax,
};
use crate::rules::{Rule, StandardDef};

pub const DEF: StandardDef = StandardDef {
    name: "ARIA in HTML",
    rules: RULES,
    rules_file: include_str!("../../../standards/html-aria-rules.md"),
    reference_url,
};

const RULES: &[Rule] = &[
    Rule::new(&["HTMLARIA002"], roles::check_redundant_semantics),
    Rule::new(&["HTMLARIA003"], roles::check_generic_role),
    Rule::new(&["HTMLARIA004"], roles::check_abstract_role),
    Rule::new(&["HTMLARIA007"], attributes::check_disallowed_aria),
    Rule::new(&["HTMLARIA009"], attributes::check_hidden_focusable),
    Rule::new(&["HTMLARIA010"], attributes::check_hidden_twice),
    Rule::new(&["HTMLARIA011"], attributes::check_advised_against_aria),
    Rule::new(&["HTMLARIA013"], native_attributes::check_conflicting_aria),
    Rule::new(&["HTMLARIA014"], native_attributes::check_repeated_aria),
    Rule::new(&["HTMLARIA015"], deprecated::check_deprecated_aria),
    Rule::new(&["HTMLARIA016"], syntax::check_lowercase_tokens),
    Rule::new(
        &["HTMLARIA017"],
        content_model::check_disallowed_descendants,
    ),
];

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
