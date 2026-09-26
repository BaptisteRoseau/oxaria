//! WCAG 1.3.1 checks for data table markup.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

/// F91: a table with data rows but no `th` cells leaves row/column headers unmarked.
pub fn check_table_missing_headers(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("table")
        .filter(|table| !is_presentation_table(*table))
        .filter(|table| has_data_rows(*table))
        .filter(|table| !has_header_cells(*table))
        .map(|_| {
            Finding::error(
                "F91",
                "table has data rows but no <th> header cells".to_string(),
            )
        })
        .collect()
}

fn is_presentation_table(table: ElementRef) -> bool {
    matches!(table.attr("role"), Some("presentation") | Some("none"))
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
        .map(|_| Finding::error("H63", "<th> has no scope attribute".to_string()))
        .collect()
}

fn referenced_header_ids(page: &RenderedPage) -> Vec<String> {
    page.select(|el| el.has_attr("headers"))
        .into_iter()
        .filter_map(|cell| cell.attr("headers"))
        .flat_map(|ids| {
            ids.split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn is_referenced(th: ElementRef, referenced_ids: &[String]) -> bool {
    match th.attr("id") {
        Some(id) => referenced_ids.iter().any(|referenced| referenced == id),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn options() -> CheckOptions {
        CheckOptions {
            contrast_threshold: 4.5,
            large_text_contrast_threshold: 3.0,
            target_size_threshold: 24.0,
        }
    }

    #[test]
    fn table_without_th_is_flagged() {
        let p = page_from_html("<table><tr><td>Month</td><td>Revenue</td></tr></table>");
        let findings = check_table_missing_headers(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "F91");
    }

    #[test]
    fn table_with_th_is_not_flagged() {
        let p = page_from_html(
            r#"<table><tr><th scope="col">Month</th></tr><tr><td>January</td></tr></table>"#,
        );
        assert!(check_table_missing_headers(&p, &options()).is_empty());
    }

    #[test]
    fn presentation_table_is_not_flagged() {
        let p =
            page_from_html(r#"<table role="presentation"><tr><td>A</td><td>B</td></tr></table>"#);
        assert!(check_table_missing_headers(&p, &options()).is_empty());
    }

    #[test]
    fn table_without_data_rows_is_not_flagged() {
        let p = page_from_html("<table><tr><th>Header only</th></tr></table>");
        assert!(check_table_missing_headers(&p, &options()).is_empty());
    }

    #[test]
    fn th_without_scope_is_flagged() {
        let p = page_from_html("<table><tr><th>Name</th></tr></table>");
        let findings = check_header_missing_scope(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "H63");
    }

    #[test]
    fn th_with_scope_is_not_flagged() {
        let p = page_from_html(r#"<table><tr><th scope="col">Name</th></tr></table>"#);
        assert!(check_header_missing_scope(&p, &options()).is_empty());
    }

    #[test]
    fn th_referenced_by_headers_attribute_is_not_flagged() {
        let p = page_from_html(
            r#"<table><tr><th id="name-h">Name</th></tr><tr><td headers="name-h">Alex</td></tr></table>"#,
        );
        assert!(check_header_missing_scope(&p, &options()).is_empty());
    }
}
