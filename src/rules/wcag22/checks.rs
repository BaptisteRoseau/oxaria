use super::{
    aria, authentication, autocomplete, contrast, document, focus, forms, frames, headings, images,
    label_in_name, language, links, multimedia, navigation, scripting, tables, target_size, timing,
};
use crate::rules::RuleCheck;

pub fn rule_checks() -> Vec<RuleCheck> {
    vec![
        images::check_missing_alt,
        images::check_non_alternative_alt,
        forms::check_missing_label,
        forms::check_required_not_indicated,
        forms::check_unnamed_control,
        headings::check_missing_h1,
        headings::check_skipped_heading_level,
        language::check_missing_lang,
        links::check_non_descriptive_link_text,
        links::check_ambiguous_duplicate_link_text,
        contrast::check_text_contrast,
        tables::check_table_missing_headers,
        tables::check_header_missing_scope,
        aria::check_duplicate_ids,
        aria::check_dangling_aria_reference,
        multimedia::check_video_missing_captions,
        multimedia::check_autoplay_without_controls,
        focus::check_outline_removed_without_alternative,
        navigation::check_missing_skip_link,
        target_size::check_target_size,
        document::check_missing_title,
        document::check_placeholder_title,
        timing::check_timed_meta_refresh,
        frames::check_untitled_iframe,
        autocomplete::check_invalid_autocomplete,
        label_in_name::check_label_not_in_name,
        tables::check_presentation_table_semantics,
        tables::check_headers_reference,
        headings::check_presentational_heading,
        scripting::check_emulated_link,
        scripting::check_control_without_role,
        scripting::check_pointer_only_handler,
        scripting::check_focus_removed_on_focus,
        multimedia::check_autoplay_audio_without_controls,
        multimedia::check_marquee,
        authentication::check_paste_blocked,
    ]
}

#[cfg(test)]
mod tests {
    use crate::rules::Standard;
    use crate::rules::registry::tests::assert_every_rule_has_help;

    #[tokio::test]
    async fn every_rule_fires_with_help() {
        let page = r##"<title>Untitled</title><style>a:focus { outline: none }</style>
                   <a href="#nowhere">Skip</a><h2>A</h2><h4>B</h4>
                   <img src="a.png"><img src="b.png" alt="image"><input required>
                   <button aria-describedby="gone"></button><div role="button"></div>
                   <a href="/a">read more</a><a href="/b">read more</a><a href="/c"></a>
                   <p style="color: #999999">low</p><button style="width: 10px; height: 10px">x</button>
                   <table><tr><td>1</td></tr></table><table><tr><th>H</th></tr></table>
                   <i id="d"></i><i id="d"></i><video autoplay></video>
                   <meta http-equiv="refresh" content="5"><meta http-equiv="refresh" content="5; url=/x">
                   <iframe src="f.html"></iframe><input autocomplete="birthday">
                   <a href="/i"><img src="i.png" alt=""></a><button aria-label="Find">Go</button>
                   <table role="presentation"><tr><th>T</th><td headers="nope">1</td></tr></table>
                   <h3 role="presentation">P</h3><span onclick="location.href='/s'">S</span>
                   <div onclick="go()">D</div><img src="n.png" alt="Next" onmousedown="next()">
                   <input type="submit" onfocus="this.blur()"><marquee>M</marquee>
                   <input type="password" onpaste="return false">"##;
        assert_every_rule_has_help(Standard::Wcag22, &[page, "<p>No title</p>"], 38).await;
    }
}
