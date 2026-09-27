//! ACT rule a25f45 for table cells' `headers` attributes.

use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{effective_role, is_hidden};
use crate::rules::{CheckOptions, Finding};

/// a25f45: each id in a cell's `headers` must be another cell of the same
/// table.
pub fn check_headers_refer_to_cells(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_cell(*el))
        .filter_map(|cell| Some((cell, cell.attr("headers")?, exposed_table(cell)?)))
        .filter_map(|(cell, headers, table)| {
            let invalid = invalid_references(cell, headers, table);
            (!invalid.is_empty()).then(|| headers_finding(cell, headers, &invalid))
        })
        .collect()
}

fn is_cell(el: ElementRef) -> bool {
    matches!(el.tag(), "td" | "th")
}

/// The cell's table, when it is exposed as a table, grid or treegrid.
fn exposed_table(cell: ElementRef) -> Option<ElementRef> {
    let table = cell
        .ancestors()
        .find(|ancestor| ancestor.tag() == "table")?;
    let is_table_role = effective_role(table)
        .is_some_and(|role| matches!(role.name, "table" | "grid" | "treegrid"));
    (is_table_role && !is_hidden(table)).then_some(table)
}

fn invalid_references<'a>(cell: ElementRef, headers: &'a str, table: ElementRef) -> Vec<&'a str> {
    headers
        .split_ascii_whitespace()
        .filter(|id| !refers_to_other_cell(cell, id, table))
        .collect()
}

fn refers_to_other_cell(cell: ElementRef, id: &str, table: ElementRef) -> bool {
    table.descendants().any(|other| {
        other != cell
            && is_cell(other)
            && other.attr("id") == Some(id)
            && exposed_table(other).is_some_and(|own_table| own_table == table)
    })
}

fn headers_finding(cell: ElementRef, headers: &str, invalid: &[&str]) -> Finding {
    let quoted: Vec<String> = invalid.iter().map(|id| format!("\"{id}\"")).collect();
    let is_self_reference = invalid.iter().all(|id| cell.attr("id") == Some(*id));
    let help = match is_self_reference {
        true => "remove the cell's own id from headers: a cell can't be its own header".to_string(),
        false => format!(
            "point headers at the id of a <th> (or other cell) in this table, or add id={} to \
             the header cell it means",
            quoted[0]
        ),
    };
    Finding::error(
        "a25f45",
        format!(
            "<{} headers=\"{headers}\"> refers to {}, not another cell of the same table",
            cell.tag(),
            quoted.join(", ")
        ),
    )
    .at(cell)
    .help(help)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;

    fn count(html: &str) -> usize {
        check_headers_refer_to_cells(&page_from_html(html), &CheckOptions::default()).len()
    }

    #[rstest]
    #[case(
        r#"<table><tr><th id="h1">Projects</th><th id="h2">Objective</th></tr>
           <tr><td headers="head1">15%</td><td headers="head2">10%</td></tr></table>"#,
        2
    )]
    #[case(
        r#"<table><tr><th id="h1">Projects</th></tr></table>
           <table><tr><td headers="h1">15%</td></tr></table>"#,
        1
    )]
    #[case(
        r#"<table><tr><th>Event</th></tr><tr><td id="b" headers="b">Birthday</td></tr></table>"#,
        1
    )]
    #[case(
        r#"<table><tr><td><span id="p">Projects</span></td></tr><tr><td headers="p">15%</td></tr></table>"#,
        1
    )]
    fn references_outside_the_tables_cells_fail(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(html), expected, "{html}");
    }

    #[rstest]
    #[case(
        r#"<table><thead><tr><th id="h1">Projects</th><th id="h2">Objective</th></tr></thead>
           <tbody><tr><td headers="h1">15%</td><td headers="h2">10%</td></tr></tbody></table>"#
    )]
    #[case(
        r#"<table><tr><th id="h1">P</th><th id="h2">E</th></tr><tr><td colspan="2" headers="h1 h2">15%</td></tr></table>"#
    )]
    #[case(
        r#"<table><tr><td role="columnheader" id="h1">P</td></tr><tr><td headers="h1">15%</td></tr></table>"#
    )]
    #[case(r#"<table><tr><th id="n" colspan="2">Name</th></tr><tr><th headers="n">First</th></tr></table>"#)]
    #[case(r#"<table><tr><th scope="col">Projects</th></tr><tr><td>15%</td></tr></table>"#)]
    #[case(
        r#"<table role="presentation"><tr><td id="h1">Status</td></tr><tr><td headers="h9">15%</td></tr></table>"#
    )]
    #[case(r#"<table hidden><tr><td headers="nope">15%</td></tr></table>"#)]
    #[case(r#"<div role="table"><div role="cell" headers="nope">15%</div></div>"#)]
    #[case(r#"<table role="heading" aria-level="1"><tr><td id="s" headers="s">World</td></tr></table>"#)]
    fn references_to_other_cells_pass(#[case] html: &str) {
        assert_eq!(count(html), 0, "{html}");
    }

    #[test]
    fn message_quotes_the_dangling_ids() {
        let page = page_from_html(
            r#"<table><tr><th id="h1">P</th></tr><tr><td headers="h1 gone">1</td></tr></table>"#,
        );
        let findings = check_headers_refer_to_cells(&page, &CheckOptions::default());
        assert_eq!(
            findings[0].message,
            r#"<td headers="h1 gone"> refers to "gone", not another cell of the same table"#
        );
    }
}
