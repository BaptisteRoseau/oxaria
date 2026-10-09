//! Finding `meta` elements by their `http-equiv` or `name`.

use crate::page::{ElementRef, RenderedPage};

/// `meta http-equiv="refresh"` elements, in document order.
pub fn refresh_metas(page: &RenderedPage) -> impl Iterator<Item = ElementRef<'_>> {
    metas(page, "http-equiv", "refresh")
}

/// `meta name="viewport"` elements, in document order.
pub fn viewport_metas(page: &RenderedPage) -> impl Iterator<Item = ElementRef<'_>> {
    metas(page, "name", "viewport")
}

fn metas<'a>(
    page: &'a RenderedPage,
    attribute: &'static str,
    value: &'static str,
) -> impl Iterator<Item = ElementRef<'a>> {
    page.by_tag("meta").filter(move |meta| {
        meta.attr(attribute)
            .is_some_and(|actual| actual.trim().eq_ignore_ascii_case(value))
    })
}
