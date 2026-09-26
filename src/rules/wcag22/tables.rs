//! WCAG 1.3.1 checks for data and layout table markup.

use std::collections::HashSet;

use crate::page::{ElementRef, RenderedPage};

use crate::rules::{CheckOptions, Finding};

/// F91: a table with data rows but no `th` cells leaves row/column headers unmarked.
pub fn check_table_missing_headers(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("table")
        .filter(|table| !is_presentation_table(*table))
        .filter(|table| has_data_rows(*table))
        .filter(|table| !has_header_cells(*table))
        .map(|table| {
            Finding::error(
                "F91",
                "table has data rows but no <th> header cells".to_string(),
            )
            .at(table)
            .help(
                "mark the header cells up as <th scope=\"col\"> (or scope=\"row\"); \
                 if the table is only for layout, add role=\"presentation\"",
            )
        })
        .collect()
}

fn is_presentation_table(table: ElementRef) -> bool {
    matches!(table.attr("role"), Some("presentation" | "none"))
}

fn has_data_rows(table: ElementRef) -> bool {
    table.descendants().any(|el| el.tag() == "td")
}

fn has_header_cells(table: ElementRef) -> bool {
    table.descendants().any(|el| el.tag() == "th")
}

/// H63: a `th` should declare `scope`, unless it is instead referenced by a `td`'s `headers`
/// attribute (the complex-table association method from H43).
pub fn check_header_missing_scope(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    let referenced_ids = referenced_header_ids(page);
    page.by_tag("th")
        .filter(|th| th.attr("scope").is_none())
        .filter(|th| !is_referenced(*th, &referenced_ids))
        .map(|th| {
            Finding::error("H63", "<th> has no scope attribute".to_string())
                .at(th)
                .help(format!(
                    "add scope=\"{}\" to say which cells this header describes",
                    likely_scope(th)
                ))
        })
        .collect()
}

/// A header in a row of only headers heads a column; one next to data cells heads its row.
fn likely_scope(th: ElementRef) -> &'static str {
    let row_has_data = th
        .ancestors()
        .find(|el| el.tag() == "tr")
        .is_some_and(|row| row.children().any(|cell| cell.tag() == "td"));
    match row_has_data {
        true => "row",
        false => "col",
    }
}

fn referenced_header_ids(page: &RenderedPage) -> HashSet<&str> {
    page.all()
        .filter_map(|cell| cell.attr("headers"))
        .flat_map(str::split_whitespace)
        .collect()
}

fn is_referenced(th: ElementRef, referenced_ids: &HashSet<&str>) -> bool {
    th.attr("id").is_some_and(|id| referenced_ids.contains(id))
}

/// F46 (layout table carrying data-table markup) / F92 (data table hidden by
/// `role="presentation"`): header cells, a caption, or a summary contradict the role, so
/// either the markup or the role misrepresents the table.
pub fn check_presentation_table_semantics(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.by_tag("table")
        .filter(|table| is_presentation_table(*table))
        .filter_map(|table| {
            let markup = data_table_markup(table);
            (!markup.is_empty()).then(|| {
                Finding::error(
                    "F46",
                    format!(
                        "table with role=\"{}\" uses data-table markup: {}",
                        table.attr("role").unwrap_or_default(),
                        markup.join(", ")
                    ),
                )
                .at(table)
                .help(
                    "if the table only lays content out, use <td> cells and drop the caption \
                     and summary; if it holds data, remove role=\"presentation\" (F92)",
                )
            })
        })
        .collect()
}

fn data_table_markup(table: ElementRef) -> Vec<&'static str> {
    let cells = own_elements(table);
    let has_tag = |tag| cells.iter().any(|el| el.tag() == tag);
    let has_attr = |attr| cells.iter().any(|el| el.has_attr(attr));
    [
        (has_tag("th"), "<th>"),
        (has_tag("caption"), "<caption>"),
        (
            table.attr("summary").is_some_and(|s| !s.trim().is_empty()),
            "summary",
        ),
        (has_attr("headers"), "headers"),
        (has_attr("scope"), "scope"),
    ]
    .into_iter()
    .filter_map(|(present, markup)| present.then_some(markup))
    .collect()
}

/// A table's elements, without those of tables nested inside it.
fn own_elements(table: ElementRef) -> Vec<ElementRef> {
    let mut found = Vec::new();
    let mut stack: Vec<_> = table.children().collect();
    while let Some(el) = stack.pop() {
        if el.tag() != "table" {
            stack.extend(el.children());
        }
        found.push(el);
    }
    found
}

/// F90: every id in a cell's `headers` must name a `th` of the same table, or the cell is
/// announced with the wrong headers (or none).
pub fn check_headers_reference(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| matches!(el.tag(), "td" | "th"))
        .filter_map(|cell| {
            let tokens = cell.attr("headers")?;
            let table = cell.ancestors().find(|el| el.tag() == "table")?;
            let header_ids = header_ids(table);
            let wrong: Vec<&str> = tokens
                .split_whitespace()
                .filter(|id| !header_ids.contains(id))
                .collect();
            (!wrong.is_empty()).then(|| {
                Finding::error(
                    "F90",
                    format!(
                        "<{}> headers=\"{tokens}\" references {} that is not a <th> of its table",
                        cell.tag(),
                        wrong
                            .iter()
                            .map(|id| format!("\"{id}\""))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
                .at(cell)
                .help(
                    "point headers at the ids of the <th> cells that head this cell, in this table",
                )
            })
        })
        .collect()
}

fn header_ids(table: ElementRef<'_>) -> HashSet<&str> {
    own_elements(table)
        .into_iter()
        .filter(|el| el.tag() == "th")
        .filter_map(|th| th.attr("id"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    #[test]
    fn scope_help_guesses_col_for_header_rows_and_row_beside_data() {
        let p = page_from_html(
            "<table><tr><th>Name</th><th>Score</th></tr><tr><th>Alex</th><td>92</td></tr></table>",
        );
        let helps: Vec<_> = check_header_missing_scope(&p, &CheckOptions::default())
            .into_iter()
            .map(|finding| finding.help.unwrap())
            .collect();
        let scopes: Vec<_> = helps
            .iter()
            .map(|help| help.split('"').nth(1).unwrap())
            .collect();
        assert_eq!(scopes, ["col", "col", "row"]);
    }

    #[test]
    fn table_without_th_is_flagged() {
        let p = page_from_html("<table><tr><td>Month</td><td>Revenue</td></tr></table>");
        let findings = check_table_missing_headers(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F91");
    }

    #[test]
    fn table_with_th_is_not_flagged() {
        let p = page_from_html(
            r#"<table><tr><th scope="col">Month</th></tr><tr><td>January</td></tr></table>"#,
        );
        assert!(check_table_missing_headers(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn presentation_table_is_not_flagged() {
        let p =
            page_from_html(r#"<table role="presentation"><tr><td>A</td><td>B</td></tr></table>"#);
        assert!(check_table_missing_headers(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn table_without_data_rows_is_not_flagged() {
        let p = page_from_html("<table><tr><th>Header only</th></tr></table>");
        assert!(check_table_missing_headers(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn th_without_scope_is_flagged() {
        let p = page_from_html("<table><tr><th>Name</th></tr></table>");
        let findings = check_header_missing_scope(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H63");
    }

    #[test]
    fn th_with_scope_is_not_flagged() {
        let p = page_from_html(r#"<table><tr><th scope="col">Name</th></tr></table>"#);
        assert!(check_header_missing_scope(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn th_referenced_by_headers_attribute_is_not_flagged() {
        let p = page_from_html(
            r#"<table><tr><th id="name-h">Name</th></tr><tr><td headers="name-h">Alex</td></tr></table>"#,
        );
        assert!(check_header_missing_scope(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn layout_table_with_th_is_flagged() {
        let p = page_from_html(
            r#"<table role="presentation"><tr><th colspan=3>Page Title</th></tr>
               <tr><td>navigation</td><td>main</td><td>sidebar</td></tr></table>"#,
        );
        let findings = check_presentation_table_semantics(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F46");
        assert!(
            findings[0].message.ends_with("markup: <th>"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn data_table_marked_presentation_lists_its_markup() {
        let p = page_from_html(
            r#"<table role="presentation" summary="Fruit colors"><caption>Fruits and their colors</caption>
               <tr><th>Name</th><th>Color</th></tr><tr><td scope="row">banana</td><td>yellow</td></tr></table>"#,
        );
        let findings = check_presentation_table_semantics(&p, &CheckOptions::default());
        assert!(
            findings[0]
                .message
                .ends_with("<th>, <caption>, summary, scope"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn plain_layout_table_is_not_flagged() {
        let p = page_from_html(
            r#"<table role="none" summary=""><tr><td>A</td><td>B</td></tr></table>"#,
        );
        assert!(check_presentation_table_semantics(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn data_table_nested_in_layout_table_is_not_flagged() {
        let p = page_from_html(
            r#"<table role="presentation"><tr><td>
               <table><tr><th scope="col">Name</th></tr><tr><td>Alex</td></tr></table>
               </td></tr></table>"#,
        );
        assert!(check_presentation_table_semantics(&p, &CheckOptions::default()).is_empty());
    }

    #[test]
    fn headers_pointing_at_a_missing_or_data_cell_is_flagged() {
        let p = page_from_html(
            r#"<table><tr><th id="h">Homework</th><td id="d">x</td></tr>
               <tr><td headers="h">15%</td><td headers="h e1 d">10%</td></tr></table>"#,
        );
        let findings = check_headers_reference(&p, &CheckOptions::default());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F90");
        assert!(
            findings[0].message.contains(r#"references "e1", "d" that"#),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn headers_pointing_at_another_tables_th_is_flagged() {
        let p = page_from_html(
            r#"<table><tr><th id="other">Other</th></tr></table>
               <table><tr><th id="h">H</th></tr><tr><td headers="other">1</td></tr></table>"#,
        );
        assert_eq!(
            check_headers_reference(&p, &CheckOptions::default()).len(),
            1
        );
    }
}
