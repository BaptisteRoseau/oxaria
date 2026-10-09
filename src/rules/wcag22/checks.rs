use super::{
    aria, authentication, autocomplete, contrast, document, focus, forms, frames, headings, images,
    label_in_name, language, links, multimedia, navigation, reference::reference_url, scripting,
    tables, target_size, timing,
};
use crate::rules::{Rule, StandardDef};

pub const DEF: StandardDef = StandardDef {
    name: "WCAG 2.2",
    rules: RULES,
    rules_file: include_str!("../../../standards/wcag2.2-rules.md"),
    reference_url,
};

const RULES: &[Rule] = &[
    Rule::new(&["F65"], images::check_missing_alt),
    Rule::new(&["F30"], images::check_non_alternative_alt),
    Rule::new(&["H44"], forms::check_missing_label),
    Rule::new(&["H90"], forms::check_required_not_indicated),
    Rule::new(&["F68"], forms::check_unnamed_control),
    Rule::new(&["H42"], headings::check_missing_h1),
    Rule::new(&["G141"], headings::check_skipped_heading_level),
    Rule::new(&["H57"], language::check_missing_lang),
    Rule::new(&["F89", "H30"], links::check_non_descriptive_link_text),
    Rule::new(&["LNK001"], links::check_ambiguous_duplicate_link_text),
    Rule::new(&["G18", "G145"], contrast::check_text_contrast),
    Rule::new(&["F91"], tables::check_table_missing_headers),
    Rule::new(&["H63"], tables::check_header_missing_scope),
    Rule::new(&["IDS001"], aria::check_duplicate_ids),
    Rule::new(&["ARIA16", "ARIA1"], aria::check_dangling_aria_reference),
    Rule::new(&["G87"], multimedia::check_video_missing_captions),
    Rule::new(&["G186"], multimedia::check_autoplay_without_controls),
    Rule::new(&["G195"], focus::check_outline_removed_without_alternative),
    Rule::new(&["G1"], navigation::check_missing_skip_link),
    Rule::new(&["TGT001"], target_size::check_target_size),
    Rule::new(&["H25"], document::check_missing_title),
    Rule::new(&["F25"], document::check_placeholder_title),
    Rule::new(&["F40", "F41"], timing::check_timed_meta_refresh),
    Rule::new(&["H64"], frames::check_untitled_iframe),
    Rule::new(&["F107"], autocomplete::check_invalid_autocomplete),
    Rule::new(&["F96"], label_in_name::check_label_not_in_name),
    Rule::new(&["F46"], tables::check_presentation_table_semantics),
    Rule::new(&["F90"], tables::check_headers_reference),
    Rule::new(&["F92"], headings::check_presentational_heading),
    Rule::new(&["F42"], scripting::check_emulated_link),
    Rule::new(&["F59"], scripting::check_control_without_role),
    Rule::new(&["F54"], scripting::check_pointer_only_handler),
    Rule::new(&["F55"], scripting::check_focus_removed_on_focus),
    Rule::new(&["F93"], multimedia::check_autoplay_audio_without_controls),
    Rule::new(&["F16"], multimedia::check_marquee),
    Rule::new(&["AUT001"], authentication::check_paste_blocked),
];

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
                   <p style="color: #999999">low</p><p style="color: #999999; font-size: 30px">big</p><button style="width: 10px; height: 10px">x</button>
                   <table><tr><td>1</td></tr></table><table><tr><th>H</th></tr></table>
                   <i id="d"></i><i id="d"></i><video autoplay></video>
                   <meta http-equiv="refresh" content="5"><meta http-equiv="refresh" content="5; url=/x">
                   <iframe src="f.html"></iframe><input autocomplete="birthday">
                   <a href="/i"><img src="i.png" alt=""></a><button aria-label="Find">Go</button>
                   <table role="presentation"><tr><th>T</th><td headers="nope">1</td></tr></table>
                   <h3 role="presentation">P</h3><span onclick="location.href='/s'">S</span>
                   <div onclick="go()">D</div><img src="n.png" alt="Next" onmousedown="next()">
                   <input type="submit" onfocus="this.blur()"><marquee>M</marquee>
                   <input type="password" onpaste="return false"><input aria-labelledby="missing">"##;
        assert_every_rule_has_help(Standard::Wcag22, &[page, "<p>No title</p>"]).await;
    }
}
