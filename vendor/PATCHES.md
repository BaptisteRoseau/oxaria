# Vendored patches

`litehtml-sys` and `litehtml` are vendored, patched copies of
[franzos/litehtml-rs](https://github.com/franzos/litehtml-rs) (MIT), pinned at
`litehtml-sys` 0.2.5 / `litehtml` 0.2.6. They are vendored rather than pulled
from crates.io so the missing accessors below can be added locally, and so the
build never needs network access.

## Why vendor at all

The upstream crates already vendor litehtml's own C++ source (see
`litehtml-sys/vendor/litehtml/`) and compile it via the `cc` crate with
pre-generated bindings, so the whole engine is statically linked into the
final binary with no runtime download and no external browser process.

## What was changed

The upstream `Element` type exposes layout data (bounding box, font size,
line height, text-align) but not tag name, attribute values, or computed
color -- everything wcag-checker needs to read DOM/ARIA attributes and
resolve text/background color. The underlying C++ engine already has all of
this internally (`element::get_tagName()`, `element::get_attr()` /
`dump_get_attrs()`, `css_properties::get_color()`,
`element::get_background()`); it just wasn't wired through the C wrapper.

Added to `litehtml-sys/csrc/litehtml_c.h` and `.cpp`:

- `lh_element_get_tag_name` -- tag name via callback (e.g. `"div"`, `"img"`).
- `lh_element_get_attr` -- value of a single named HTML attribute (e.g.
  `"href"`, `"aria-label"`) via callback, wrapping `element::get_attr()`.
  litehtml's `dump_get_attrs()` looks like it would enumerate attributes,
  but it actually dumps computed
  CSS properties for its own debug tree-printer, not HTML attributes (verified
  empirically, not just by reading the header); `lh_element_for_each_attr`
  below is the real enumerator.
- `lh_element_for_each_attr` -- every HTML attribute of an element (name
  lowercased, value as parsed by gumbo) via one callback per attribute. The
  attributes live in `html_tag::m_attrs`, which is `protected`, so
  `litehtml-sys/vendor/litehtml/include/litehtml/html_tag.h` (the engine itself, not just
  the wrapper) got a one-line public getter, `attrs()`; the wrapper reaches it
  with a `dynamic_cast<html_tag*>` (text and comment nodes aren't `html_tag`s
  and yield nothing). litehtml also writes one attribute of its own there,
  `list_index` on list items during layout, which wcag-checker filters out.
- `lh_element_get_color` -- computed CSS `color`, resolved through inheritance.
- `lh_element_get_background_color` -- the element's own declared
  `background-color` (backgrounds don't inherit in CSS, so callers wanting
  the effective visual background walk up via `lh_element_parent` themselves
  until a non-transparent color or the root is reached).

Matching declarations were hand-added to the pre-generated
`litehtml-sys/src/bindings.rs` (the crate's default build skips `bindgen`
entirely and just copies this file, so no libclang is needed to build), and
matching safe methods (`tag_name`, `attr`, `attrs`, `color`,
`background_color`) were added to `litehtml/src/lib.rs`'s `Element` impl.

`litehtml/Cargo.toml`'s `litehtml-sys` dependency was changed from a
crates.io version to `path = "../litehtml-sys"`.

`Element<'a>` was given `#[derive(Clone, Copy)]` -- it only ever wraps a
borrowed pointer (like any other `&_`), so duplicating it is exactly as safe
as copying a reference; all unsafety stays contained within the methods that
dereference it. This lets wcag-checker walk up an element's ancestor chain
(`while let Some(parent) = el.parent() { el = parent; }`) without threading
lifetimes through a recursive helper.

## Known limitation carried into wcag-checker

litehtml has no `:focus`/`:hover`/`:active` state-simulation API, so nothing
here can compare an element's focused vs. unfocused computed style. The
focus-outline rule (G195) keeps its own `<style>`-text-scanning heuristic
rather than relying on the engine for that specific check.
