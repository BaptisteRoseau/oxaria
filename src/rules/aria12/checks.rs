use super::{
    attributes, deprecated, focus, idrefs, keyshortcuts, landmarks, presentation,
    reference::reference_url, roles, sets, structure, usage, values, widgets,
};
use crate::rules::{Rule, StandardDef};

pub const DEF: StandardDef = StandardDef {
    name: "WAI-ARIA 1.2",
    rules: RULES,
    rules_file: include_str!("../../../standards/wai-aria-1.2-rules.md"),
    reference_url,
};

const RULES: &[Rule] = &[
    Rule::new(&["ARIA-ROLE002"], roles::check_abstract_role),
    Rule::new(&["ARIA-ROLE003"], roles::check_explicit_generic),
    Rule::new(&["ARIA-ATTR001"], attributes::check_unknown_attribute),
    Rule::new(&["ARIA-ATTR003"], attributes::check_unsupported_attribute),
    Rule::new(&["ARIA-ATTR004"], attributes::check_prohibited_attribute),
    Rule::new(&["ARIA-ATTR005"], attributes::check_static_table_attribute),
    Rule::new(&["ARIA-ATTR007"], keyshortcuts::check_keyshortcuts_syntax),
    Rule::new(&["ARIA-ATTR008"], attributes::check_role_description),
    Rule::new(&["ARIA-ATTR009"], sets::check_multiple_current_or_sorted),
    Rule::new(&["ARIA-VAL001"], values::check_value_type),
    Rule::new(&["ARIA-VAL002"], values::check_integer_range),
    Rule::new(&["ARIA-VAL003"], values::check_range_consistency),
    Rule::new(&["ARIA-VAL005"], values::check_missing_row_index),
    Rule::new(&["ARIA-IDREF001"], idrefs::check_unresolved_reference),
    Rule::new(&["ARIA-IDREF002"], idrefs::check_aria_owns_conflict),
    Rule::new(
        &["ARIA-IDREF003"],
        idrefs::check_active_descendant_ownership,
    ),
    Rule::new(&["ARIA-STRUCT001"], structure::check_required_context),
    Rule::new(&["ARIA-STRUCT002"], structure::check_required_owned),
    Rule::new(&["ARIA-STRUCT003"], structure::check_restricted_children),
    Rule::new(&["ARIA-STRUCT004"], structure::check_caption_placement),
    Rule::new(&["ARIA-PRES001"], presentation::check_ignored_presentation),
    Rule::new(
        &["ARIA-PRES002"],
        presentation::check_presentational_image_alt,
    ),
    Rule::new(
        &["ARIA-PRES003"],
        presentation::check_content_in_presentational_children,
    ),
    Rule::new(&["ARIA-WIDGET001"], widgets::check_combobox_pattern),
    Rule::new(&["ARIA-WIDGET002"], widgets::check_popup_role_match),
    Rule::new(&["ARIA-WIDGET003"], widgets::check_autocomplete_popup),
    Rule::new(
        &["ARIA-WIDGET006"],
        widgets::check_unfocusable_popup_trigger,
    ),
    Rule::new(&["ARIA-WIDGET007"], widgets::check_unfocusable_feed_article),
    Rule::new(&["ARIA-FOCUS001"], focus::check_unfocusable_widget),
    Rule::new(&["ARIA-FOCUS006"], focus::check_dialog_without_focusable),
    Rule::new(&["ARIA-LMK001"], landmarks::check_duplicate_landmark),
    Rule::new(&["ARIA-USAGE002"], usage::check_role_purpose),
    Rule::new(&["ARIA-USAGE003"], usage::check_naming_mechanism),
    Rule::new(&["ARIA-DEPR001"], deprecated::check_directory_role),
    Rule::new(
        &["ARIA-DEPR002"],
        deprecated::check_drag_and_drop_attributes,
    ),
    Rule::new(
        &["ARIA-DEPR003"],
        deprecated::check_deprecated_global_attributes,
    ),
];

/// Pages that, together, make every rule above fire.
#[cfg(test)]
pub(crate) const HELP_FIXTURES: &[&str] = &[
    r#"<div role="range">r</div><section role="generic">g</section>
            <input aria-labeledby="x"><div role="button" tabindex="0" aria-checked="true">b</div>
            <p aria-label="Warning">w</p><table><tr><td aria-colspan="2">t</td></tr></table>
            <button aria-keyshortcuts="Ctrl+S">s</button><svg aria-roledescription="chart"></svg>
            <nav><a href="/1" aria-current="page">1</a><a href="/2" aria-current="page">2</a></nav>
            <ul role="directory"><li>d</li></ul><li draggable="true" aria-grabbed="false">g</li>
            <section aria-disabled="true">s</section>"#,
    r#"<button aria-expanded="yes">e</button>
            <h2 aria-level="0">l</h2>
            <div role="meter" aria-label="Disk" aria-valuenow="130">130%</div>
            <div role="grid"><div role="row" aria-rowindex="1"><div role="gridcell">a</div></div>
              <div role="row"><div role="gridcell">b</div></div></div>
            <input aria-describedby="missing">
            <div role="tree" aria-owns="n7">t</div><div role="group" aria-owns="n7">g</div>
            <div role="treeitem" id="n7">n</div>
            <ul role="listbox" tabindex="0" aria-activedescendant="z"><li role="option">a</li></ul>
            <li role="option" id="z">z</li>"#,
    r#"<div role="tab" tabindex="0">t</div>
            <div role="tablist"><button>General</button></div>
            <ul role="listbox"><li role="group"><span role="heading" aria-level="3">H</span>
              <span role="option">L</span></li></ul>
            <div role="table"><div role="rowgroup"><div role="row"><div role="cell">c</div></div></div>
              <div role="caption">C</div></div>
            <a href="/c" role="presentation">c</a>
            <img src="s.png" role="presentation" alt="Sales grew">
            <div role="button" tabindex="0"><h3>Pro</h3></div>"#,
    r#"<div role="combobox" aria-expanded="false"><input aria-label="Country"></div>
            <button aria-haspopup="true" aria-controls="dlg">Share</button>
            <div role="dialog" id="dlg" aria-label="Share"><button>Close</button></div>
            <input type="search" aria-label="Search" aria-autocomplete="list">
            <span aria-haspopup="menu">Account</span>
            <div role="feed"><article>a</article></div>
            <span role="link">l</span>
            <div role="dialog" aria-label="Info"><p>Static</p></div>
            <main>m</main><div role="main">m2</div>
            <span role="time">Sept 27</span>
            <div role="alertdialog" aria-label="Delete"><button>OK</button></div>"#,
];
