//! WCAG 1.3.1 checks for data table markup.

use std::collections::HashSet;

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

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
}
