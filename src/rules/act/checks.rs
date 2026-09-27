use super::{
    aria, autocomplete, contrast, controls, focus, images, language, orientation, refresh, spacing,
    tables, zoom,
};
use crate::rules::RuleCheck;

pub fn rule_checks() -> Vec<RuleCheck> {
    vec![
        language::check_page_title,
        language::check_html_lang,
        language::check_html_lang_valid,
        language::check_element_lang_valid,
        images::check_image_name,
        images::check_decorative_exposed,
        images::check_image_button_name,
        images::check_svg_image_name,
        images::check_object_name,
        controls::check_button_name,
        controls::check_link_name,
        controls::check_menuitem_name,
        controls::check_summary_name,
        controls::check_form_field_name,
        autocomplete::check_autocomplete_valid,
        tables::check_headers_refer_to_cells,
        aria::check_aria_attribute_defined,
        aria::check_aria_value_valid,
        aria::check_role_valid,
        focus::check_focusable_in_presentational_children,
        contrast::check_contrast_minimum,
        refresh::check_refresh_delay,
        refresh::check_refresh_instant,
        zoom::check_viewport_zoom,
        spacing::check_important_letter_spacing,
        spacing::check_important_word_spacing,
        orientation::check_orientation_lock,
    ]
}

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
        assert_every_rule_has_help(Standard::Act, &[page, "<p>No title</p>"], 27).await;
    }
}
