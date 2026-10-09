use super::{
    aria, autocomplete, contrast, controls, focus, images, language, orientation, refresh, spacing,
    tables, zoom,
};
use crate::rules::Rule;

pub const RULES: &[Rule] = &[
    Rule::new(&["2779a5"], language::check_page_title),
    Rule::new(&["b5c3f8"], language::check_html_lang),
    Rule::new(&["bf051a"], language::check_html_lang_valid),
    Rule::new(&["de46e4"], language::check_element_lang_valid),
    Rule::new(&["23a2a8"], images::check_image_name),
    Rule::new(&["46ca7f"], images::check_decorative_exposed),
    Rule::new(&["59796f"], images::check_image_button_name),
    Rule::new(&["7d6734"], images::check_svg_image_name),
    Rule::new(&["8fc3b6"], images::check_object_name),
    Rule::new(&["97a4e1"], controls::check_button_name),
    Rule::new(&["c487ae"], controls::check_link_name),
    Rule::new(&["m6b1q3"], controls::check_menuitem_name),
    Rule::new(&["2t702h"], controls::check_summary_name),
    Rule::new(&["e086e5"], controls::check_form_field_name),
    Rule::new(&["73f2c2"], autocomplete::check_autocomplete_valid),
    Rule::new(&["a25f45"], tables::check_headers_refer_to_cells),
    Rule::new(&["5f99a7"], aria::check_aria_attribute_defined),
    Rule::new(&["6a7281"], aria::check_aria_value_valid),
    Rule::new(&["674b10"], aria::check_role_valid),
    Rule::new(
        &["307n5z"],
        focus::check_focusable_in_presentational_children,
    ),
    Rule::new(&["afw4f7"], contrast::check_contrast_minimum),
    Rule::new(&["bc659a"], refresh::check_refresh_delay),
    Rule::new(&["bisz58"], refresh::check_refresh_instant),
    Rule::new(&["b4f0c3"], zoom::check_viewport_zoom),
    Rule::new(&["24afc2"], spacing::check_important_letter_spacing),
    Rule::new(&["9e45ec"], spacing::check_important_word_spacing),
    Rule::new(&["b33eff"], orientation::check_orientation_lock),
];

#[cfg(test)]
mod tests {
    use crate::rules::Standard;
    use crate::rules::registry::tests::assert_every_rule_has_help;

    #[tokio::test]
    async fn every_rule_fires_with_help() {
        let page = r#"<html lang="eng"><head>
                   <meta http-equiv="refresh" content="30">
                   <meta name="viewport" content="user-scalable=no">
                   <style>@media (orientation: portrait) { html { rotate: 90deg } }</style>
                   </head><body>
                   <p lang="dutch">Zij liepen een vreemde Tiki bar binnen.</p>
                   <img src="logo.png"><img src="deco.png" alt="" aria-label="Logo">
                   <input type="image" src="search.svg"><svg role="img"></svg>
                   <object data="speech.mp3"></object>
                   <button></button><a href="/"></a>
                   <div role="menu"><div role="menuitem"></div></div>
                   <details><summary></summary><p>Open</p></details>
                   <input autocomplete="badname">
                   <table><tr><th id="h">P</th></tr><tr><td headers="nope">1</td></tr></table>
                   <div aria-labelled="x" aria-expanded="collapsed" role="lnik">x</div>
                   <p role="checkbox" aria-checked="false" tabindex="0">I agree <a href="/t">terms</a></p>
                   <p style="color: #999999; letter-spacing: 0.1em !important; word-spacing: 0 !important">Low contrast</p>
                   </body></html>"#;
        assert_every_rule_has_help(Standard::Act, &[page, "<p>No title</p>"]).await;
    }
}
