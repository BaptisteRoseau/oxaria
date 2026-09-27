use super::{
    attributes, deprecated, focus, hidden, idrefs, keyshortcuts, landmarks, presentation, roles,
    sets, structure, usage, values, widgets,
};
use crate::rules::RuleCheck;

pub fn rule_checks() -> Vec<RuleCheck> {
    vec![
        roles::check_abstract_role,
        roles::check_explicit_generic,
        attributes::check_unknown_attribute,
        attributes::check_unsupported_attribute,
        attributes::check_prohibited_attribute,
        attributes::check_static_table_attribute,
        keyshortcuts::check_keyshortcuts_syntax,
        attributes::check_role_description,
        sets::check_multiple_current_or_sorted,
        values::check_value_type,
        values::check_integer_range,
        values::check_range_consistency,
        values::check_missing_row_index,
        idrefs::check_unresolved_reference,
        idrefs::check_aria_owns_conflict,
        idrefs::check_active_descendant_ownership,
        structure::check_required_context,
        structure::check_required_owned,
        structure::check_restricted_children,
        structure::check_caption_placement,
        presentation::check_ignored_presentation,
        presentation::check_presentational_image_alt,
        presentation::check_content_in_presentational_children,
        hidden::check_hidden_content,
        widgets::check_combobox_pattern,
        widgets::check_popup_role_match,
        widgets::check_autocomplete_popup,
        widgets::check_unfocusable_popup_trigger,
        widgets::check_unfocusable_feed_article,
        focus::check_unfocusable_widget,
        focus::check_dialog_without_focusable,
        landmarks::check_duplicate_landmark,
        usage::check_role_purpose,
        usage::check_naming_mechanism,
        deprecated::check_directory_role,
        deprecated::check_drag_and_drop_attributes,
        deprecated::check_deprecated_global_attributes,
    ]
}

#[cfg(test)]
mod tests {
    use crate::rules::Standard;
    use crate::rules::registry::tests::assert_every_rule_has_help;

    #[tokio::test]
    async fn every_rule_fires_with_help() {
        let roles_and_attributes = r#"<div role="range">r</div><section role="generic">g</section>
            <input aria-labeledby="x"><div role="button" tabindex="0" aria-checked="true">b</div>
            <p aria-label="Warning">w</p><table><tr><td aria-colspan="2">t</td></tr></table>
            <button aria-keyshortcuts="Ctrl+S">s</button><svg aria-roledescription="chart"></svg>
            <nav><a href="/1" aria-current="page">1</a><a href="/2" aria-current="page">2</a></nav>
            <ul role="directory"><li>d</li></ul><li draggable="true" aria-grabbed="false">g</li>
            <section aria-disabled="true">s</section>"#;
        let values_and_references = r#"<button aria-expanded="yes">e</button>
            <h2 aria-level="0">l</h2>
            <div role="meter" aria-label="Disk" aria-valuenow="130">130%</div>
            <div role="grid"><div role="row" aria-rowindex="1"><div role="gridcell">a</div></div>
              <div role="row"><div role="gridcell">b</div></div></div>
            <input aria-describedby="missing">
            <div role="tree" aria-owns="n7">t</div><div role="group" aria-owns="n7">g</div>
            <div role="treeitem" id="n7">n</div>
            <ul role="listbox" tabindex="0" aria-activedescendant="z"><li role="option">a</li></ul>
            <li role="option" id="z">z</li>"#;
        let structure = r#"<div role="tab" tabindex="0">t</div>
            <div role="tablist"><button>General</button></div>
            <ul role="listbox"><li role="group"><span role="heading" aria-level="3">H</span>
              <span role="option">L</span></li></ul>
            <div role="table"><div role="rowgroup"><div role="row"><div role="cell">c</div></div></div>
              <div role="caption">C</div></div>
            <a href="/c" role="presentation">c</a>
            <img src="s.png" role="presentation" alt="Sales grew">
            <div role="button" tabindex="0"><h3>Pro</h3></div>
            <div aria-hidden="true"><a href="/h">h</a></div>"#;
        let widgets = r#"<div role="combobox" aria-expanded="false"><input aria-label="Country"></div>
            <button aria-haspopup="true" aria-controls="dlg">Share</button>
            <div role="dialog" id="dlg" aria-label="Share"><button>Close</button></div>
            <input type="search" aria-label="Search" aria-autocomplete="list">
            <span aria-haspopup="menu">Account</span>
            <div role="feed"><article>a</article></div>
            <span role="link">l</span>
            <div role="dialog" aria-label="Info"><p>Static</p></div>
            <main>m</main><div role="main">m2</div>
            <span role="time">Sept 27</span>
            <div role="alertdialog" aria-label="Delete"><button>OK</button></div>"#;
        assert_every_rule_has_help(
            Standard::Aria12,
            &[
                roles_and_attributes,
                values_and_references,
                structure,
                widgets,
            ],
            37,
        )
        .await;
    }
}
