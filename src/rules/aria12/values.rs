//! Attribute values: their type (ARIA-VAL001), integer ranges
//! (ARIA-VAL002), range widget consistency (ARIA-VAL003) and row indexes
//! (ARIA-VAL005).

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    AriaValue, Attribute, Parsed, Role, ValueType, aria_attributes, aria_number, aria_value,
    attribute, first_valid_role, is_focusable, is_hidden, parse_value,
};
use crate::rules::{CheckOptions, Finding};

use super::support::{closest, has_value, or_list, quoted};
use super::tree::{exposed_role, has_exposed_role};

const TABLE_ROLES: &[&str] = &["table", "grid", "treegrid"];
const RANGE_ROLES: &[&str] = &[
    "meter",
    "progressbar",
    "scrollbar",
    "separator",
    "slider",
    "spinbutton",
];
/// Range roles whose value is always known, so `aria-valuetext` needs an
/// `aria-valuenow` to go with it.
const KNOWN_VALUE_ROLES: &[&str] = &["meter", "scrollbar", "separator", "slider"];

/// ARIA-VAL001: a value of the wrong type is ignored or misread.
pub fn check_value_type(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            aria_attributes(el)
                .filter_map(|(name, value)| Some((attribute(name)?, value)))
                .filter(|(attribute, value)| parse_value(attribute, value) == Parsed::Invalid)
                .map(move |(attribute, value)| {
                    Finding::error(
                        "ARIA-VAL001",
                        format!(
                            "{} is not a valid {} value",
                            quoted(attribute.name, value),
                            attribute.name
                        ),
                    )
                    .at(el)
                    .help(value_type_help(attribute, value))
                })
        })
        .collect()
}

fn value_type_help(attribute: &Attribute, value: &str) -> String {
    let allowed = || {
        let quoted: Vec<String> = attribute
            .values
            .iter()
            .map(|v| format!("\"{v}\""))
            .collect();
        or_list(&quoted.iter().map(String::as_str).collect::<Vec<_>>())
    };
    match attribute.value_type {
        ValueType::Integer => "use a whole number, e.g. \"2\"".to_string(),
        ValueType::Number => "use a number, e.g. \"42\" or \"0.5\"".to_string(),
        ValueType::IdRef => format!(
            "use the id of a single element; {} can't reference several",
            attribute.name
        ),
        ValueType::TokenList => format!("use one or more of {}, separated by spaces", allowed()),
        _ => match closest(
            &value.trim().to_ascii_lowercase(),
            attribute.values.iter().copied(),
        ) {
            Some(suggestion) => format!("did you mean \"{suggestion}\"? use {}", allowed()),
            None => format!("use {}", allowed()),
        },
    }
}

/// ARIA-VAL002: levels, positions, counts, indexes and spans have lower
/// bounds, and positions and indexes can't exceed the size they're in.
pub fn check_integer_range(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .flat_map(|el| {
            integer_problems(el)
                .into_iter()
                .map(move |(message, help)| {
                    Finding::error("ARIA-VAL002", message).at(el).help(help)
                })
        })
        .collect()
}

fn integer_problems(el: ElementRef) -> Vec<(String, String)> {
    [
        lower_bound(el, "aria-level", 1),
        lower_bound(el, "aria-posinset", 1),
        lower_bound(el, "aria-colindex", 1),
        lower_bound(el, "aria-rowindex", 1),
        lower_bound(el, "aria-colspan", 1),
        lower_bound(el, "aria-rowspan", 0),
        count_bound(el, "aria-setsize"),
        count_bound(el, "aria-colcount"),
        count_bound(el, "aria-rowcount"),
        position_within_set(el),
        index_within_table(el, "aria-colindex", "aria-colcount"),
        index_within_table(el, "aria-rowindex", "aria-rowcount"),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn integer(el: ElementRef, name: &str) -> Option<i64> {
    match aria_value(el, name)? {
        Parsed::Valid(AriaValue::Integer(value)) => Some(value),
        _ => None,
    }
}

fn lower_bound(el: ElementRef, name: &str, minimum: i64) -> Option<(String, String)> {
    let value = integer(el, name).filter(|value| *value < minimum)?;
    Some((
        format!("{name}=\"{value}\" is below its minimum of {minimum}"),
        format!("set {name} to {minimum} or more"),
    ))
}

/// A count is the number of items, or -1 when it is unknown.
fn count_bound(el: ElementRef, name: &str) -> Option<(String, String)> {
    let value = integer(el, name).filter(|value| *value < 1 && *value != -1)?;
    Some((
        format!("{name}=\"{value}\" is neither a count nor -1"),
        format!("set {name} to the total number of items, or to -1 if it is unknown"),
    ))
}

fn position_within_set(el: ElementRef) -> Option<(String, String)> {
    let position = integer(el, "aria-posinset")?;
    let size = integer(el, "aria-setsize").filter(|size| *size >= 1 && position > *size)?;
    Some((
        format!("aria-posinset=\"{position}\" is past the end of a set of aria-setsize=\"{size}\""),
        format!("set aria-posinset between 1 and {size}, or fix aria-setsize"),
    ))
}

fn index_within_table(el: ElementRef, index: &str, count: &str) -> Option<(String, String)> {
    let value = integer(el, index)?;
    let total = el
        .ancestors()
        .filter(|ancestor| has_exposed_role(*ancestor, TABLE_ROLES))
        .find_map(|table| integer(table, count))
        .filter(|total| *total >= 1 && value > *total)?;
    Some((
        format!("{index}=\"{value}\" is past the table's {count}=\"{total}\""),
        format!("set {index} between 1 and {total}, or fix the table's {count}"),
    ))
}

/// ARIA-VAL003: a range widget's minimum, maximum and value must agree,
/// with the role's defaults for the ones left out.
pub fn check_range_consistency(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter_map(|el| Some((el, explicit_range_role(el)?)))
        .flat_map(|(el, role)| {
            range_problems(el, role)
                .into_iter()
                .map(move |(message, help)| {
                    Finding::error("ARIA-VAL003", message).at(el).help(help)
                })
        })
        .collect()
}

/// Native `meter`, `progress` and `input` use `min`/`max`, not these
/// attributes, and a separator is a range only when focusable.
fn explicit_range_role(el: ElementRef) -> Option<&'static Role> {
    let role = first_valid_role(el).filter(|role| RANGE_ROLES.contains(&role.name))?;
    let is_range = exposed_role(el).is_some_and(|exposed| exposed.name == role.name)
        && (role.name != "separator" || is_focusable(el));
    is_range.then_some(role)
}

fn range_problems(el: ElementRef, role: &Role) -> Vec<(String, String)> {
    let bound = |name: &str| {
        aria_number(el, name).or_else(|| role.default_value(name).and_then(|v| v.parse().ok()))
    };
    let (min, max) = (bound("aria-valuemin"), bound("aria-valuemax"));
    let now = aria_number(el, "aria-valuenow");
    [
        inverted_bounds(min, max),
        value_out_of_bounds(now, min, max),
        text_without_value(el, role, now),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn inverted_bounds(min: Option<f64>, max: Option<f64>) -> Option<(String, String)> {
    let (min, max) = (min?, max?);
    (max < min).then(|| {
        (
            format!("aria-valuemax ({max}) is below aria-valuemin ({min})"),
            "swap the values: aria-valuemin is the lowest value, aria-valuemax the highest"
                .to_string(),
        )
    })
}

fn value_out_of_bounds(
    now: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
) -> Option<(String, String)> {
    let (now, min, max) = (now?, min?, max?);
    (min <= max && !(min..=max).contains(&now)).then(|| {
        (
            format!("aria-valuenow ({now}) is outside the range {min} to {max}"),
            format!(
                "keep aria-valuenow between {min} and {max}, or set aria-valuemin/aria-valuemax \
                 to the real range (they default to 0 and 100)"
            ),
        )
    })
}

fn text_without_value(el: ElementRef, role: &Role, now: Option<f64>) -> Option<(String, String)> {
    let is_missing_value =
        KNOWN_VALUE_ROLES.contains(&role.name) && has_value(el, "aria-valuetext") && now.is_none();
    is_missing_value.then(|| {
        (
            format!(
                "{} is set without aria-valuenow",
                quoted(
                    "aria-valuetext",
                    el.attr("aria-valuetext").unwrap_or_default()
                )
            ),
            format!(
                "add the numeric value in aria-valuenow; a {} always has one, and aria-valuetext \
                 only describes it",
                role.name
            ),
        )
    })
}

/// ARIA-VAL005: once some rows of a table carry `aria-rowindex`, the
/// others must too, or their position in the full table is unknown.
pub fn check_missing_row_index(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let rows: Vec<(ElementRef, ElementRef)> = page
        .all()
        .filter(|el| has_exposed_role(*el, &["row"]) && !is_hidden(*el))
        .filter_map(|row| Some((row, table_of(row)?)))
        .collect();
    let indexed_tables: Vec<ElementRef> = rows
        .iter()
        .filter(|(row, _)| has_value(*row, "aria-rowindex"))
        .map(|(_, table)| *table)
        .collect();
    rows.iter()
        .filter(|(row, table)| !has_value(*row, "aria-rowindex") && indexed_tables.contains(table))
        .map(|(row, _)| {
            Finding::error(
                "ARIA-VAL005",
                "row has no aria-rowindex, although other rows of its table have one".to_string(),
            )
            .at(*row)
            .help("add aria-rowindex with the row's position in the full table, counting from 1")
        })
        .collect()
}

fn table_of(el: ElementRef) -> Option<ElementRef> {
    el.ancestors()
        .find(|ancestor| has_exposed_role(*ancestor, TABLE_ROLES))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn run(check: fn(&RenderedPage, &CheckOptions) -> Vec<Finding>, html: &str) -> Vec<Finding> {
        check(&page_from_html(html), &CheckOptions::default())
    }

    #[rstest]
    #[case(
        r#"<button aria-expanded="yes">Options</button>"#,
        "use \"false\", \"true\" or \"undefined\""
    )]
    #[case(
        r#"<div aria-live="rude">...</div>"#,
        "use \"assertive\", \"off\" or \"polite\""
    )]
    #[case(r#"<button aria-haspopup="popup">More</button>"#, "use ")]
    #[case(r#"<div aria-live="polit">...</div>"#, "did you mean \"polite\"?")]
    #[case(r#"<h2 aria-level="2.5">x</h2>"#, "use a whole number")]
    #[case(
        r#"<input aria-activedescendant="a b">"#,
        "use the id of a single element"
    )]
    #[case(
        r#"<div aria-relevant="additions bogus">x</div>"#,
        "use one or more of"
    )]
    fn invalid_values_are_flagged(#[case] html: &str, #[case] help: &str) {
        let findings = run(check_value_type, html);
        assert_eq!(findings.len(), 1, "{html}");
        assert!(
            findings[0].help.as_deref().unwrap().contains(help),
            "{findings:?}"
        );
    }

    #[rstest]
    #[case(r#"<button aria-expanded="true">Options</button>"#)]
    #[case(r#"<div aria-live="assertive">...</div>"#)]
    #[case(r#"<button aria-haspopup="menu">More</button>"#)]
    #[case(r#"<button aria-expanded="">Options</button>"#)]
    #[case(r#"<button aria-pressed="MIXED">Bold</button>"#)]
    #[case(r#"<div aria-bogus="x">x</div>"#)]
    fn valid_values_are_not_flagged(#[case] html: &str) {
        assert!(run(check_value_type, html).is_empty(), "{html}");
    }

    #[test]
    fn positions_outside_the_set_are_flagged() {
        let html = r#"<ul role="listbox" aria-label="Available fruit">
            <li role="option" aria-setsize="16" aria-posinset="0">apples</li>
            <li role="option" aria-setsize="16" aria-posinset="17">bananas</li></ul>"#;
        assert_eq!(run(check_integer_range, html).len(), 2);
    }

    #[rstest]
    #[case(r#"<h2 aria-level="0">x</h2>"#)]
    #[case(r#"<div role="grid" aria-rowcount="0"></div>"#)]
    #[case(r#"<li role="option" aria-setsize="-2">x</li>"#)]
    #[case(r#"<div role="grid" aria-colcount="3"><div role="row"><div role="gridcell" aria-colindex="4">x</div></div></div>"#)]
    #[case(r#"<div role="cell" aria-rowspan="-1">x</div>"#)]
    fn out_of_range_integers_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_integer_range, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<ul role="listbox" aria-label="Fruit"><li role="option" aria-setsize="16" aria-posinset="5">a</li>
              <li role="option" aria-setsize="16" aria-posinset="6">b</li></ul>"#)]
    #[case(r#"<li role="option" aria-setsize="-1" aria-posinset="40">x</li>"#)]
    #[case(r#"<div role="grid" aria-colcount="-1"><div role="row"><div role="gridcell" aria-colindex="40">x</div></div></div>"#)]
    #[case(r#"<div role="cell" aria-rowspan="0">x</div>"#)]
    fn in_range_integers_are_not_flagged(#[case] html: &str) {
        assert!(run(check_integer_range, html).is_empty(), "{html}");
    }

    #[rstest]
    #[case(r#"<div role="slider" tabindex="0" aria-label="Volume" aria-valuemin="10" aria-valuemax="0" aria-valuenow="50"></div>"#)]
    #[case(r#"<div role="meter" aria-label="Disk usage" aria-valuenow="130">130%</div>"#)]
    #[case(r#"<div role="slider" tabindex="0" aria-label="Size" aria-valuetext="medium"></div>"#)]
    #[case(r#"<div role="separator" tabindex="0" aria-valuenow="120"></div>"#)]
    fn inconsistent_ranges_are_flagged(#[case] html: &str) {
        assert_eq!(run(check_range_consistency, html).len(), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<div role="slider" tabindex="0" aria-label="Volume" aria-valuemin="0" aria-valuemax="10" aria-valuenow="5"></div>"#)]
    #[case(r#"<div role="meter" aria-label="Disk usage" aria-valuenow="65">65%</div>"#)]
    #[case(r#"<div role="spinbutton" tabindex="0" aria-valuenow="500"></div>"#)]
    #[case(r#"<div role="progressbar" aria-valuetext="Loading"></div>"#)]
    #[case(r#"<div role="separator" aria-valuenow="120"></div>"#)]
    #[case(r#"<meter aria-valuenow="130" value="130" max="200"></meter>"#)]
    fn consistent_ranges_are_not_flagged(#[case] html: &str) {
        assert!(run(check_range_consistency, html).is_empty(), "{html}");
    }

    #[test]
    fn rows_missing_an_index_are_flagged() {
        let html = r#"<div role="grid" aria-rowcount="2000">
            <div role="row" aria-rowindex="1"><div role="gridcell">a</div></div>
            <div role="row"><div role="gridcell">b</div></div></div>"#;
        assert_eq!(run(check_missing_row_index, html).len(), 1);
    }

    #[rstest]
    #[case(
        r#"<div role="grid" aria-rowcount="2000">
            <div role="row" aria-rowindex="1"><div role="gridcell">a</div></div>
            <div role="row" aria-rowindex="100"><div role="gridcell">b</div></div></div>"#
    )]
    #[case("<table><tr><td>a</td></tr><tr><td>b</td></tr></table>")]
    #[case(
        r#"<table><tr aria-rowindex="1"><td>a</td></tr></table><table><tr><td>b</td></tr></table>"#
    )]
    fn consistently_indexed_rows_are_not_flagged(#[case] html: &str) {
        assert!(run(check_missing_row_index, html).is_empty(), "{html}");
    }
}
