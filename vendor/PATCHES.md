# Vendored patches

oxaria embeds a patched copy of the `litehtml-sys` 0.2.5 / `litehtml` 0.2.6
crates of [franzos/litehtml-rs](https://github.com/franzos/litehtml-rs) (MIT,
see `litehtml-rs-LICENSE`), so the missing accessors below can be added
locally, and so the build never needs network access.

## Why vendor at all

The upstream crates already vendor litehtml's own C++ source and compile it
via the `cc` crate with pre-generated bindings, so the whole engine is
statically linked into the final binary with no runtime download and no
external browser process.

## Layout: folded into oxaria, not separate crates

The copies used to be two path dependencies, `vendor/litehtml-sys` and
`vendor/litehtml`. crates.io refuses path-only dependencies, and publishing
the patched copies under new names would mean three crates to release on
every change, so they were folded into oxaria itself:

| Upstream                              | Here                       |
| ------------------------------------- | -------------------------- |
| `litehtml-sys/vendor/litehtml/`       | `vendor/litehtml/` (C++ engine and gumbo, unchanged but for `html_tag.h` below) |
| `litehtml-sys/csrc/`                  | `vendor/litehtml_c/` (C wrapper) |
| `litehtml-sys/build.rs`               | `build.rs` (vendored build only: the system-library and bindgen paths are gone) |
| `litehtml-sys/src/bindings.rs`        | `src/litehtml/sys.rs`      |
| `litehtml/src/lib.rs`                 | `src/litehtml/engine.rs`   |

`engine.rs` dropped the optional `pixbuf`/`html`/`email` features and the
`selection` module (none used), logs through `tracing` instead of `log`, and
allows `unsafe_op_in_unsafe_fn` and `dead_code` rather than being rewritten:
it's edition 2021 code compiled as edition 2024, and oxaria uses part of its
API. Keep it close to upstream so the two can still be diffed. Only
`vendor/litehtml/{include,src,LICENSE}` are published (`include` in
Cargo.toml); its `containers/`, `doc/` and `support/` aren't built.

## What was changed

The upstream `Element` type exposes layout data (bounding box, font size,
line height, text-align) but not tag name, attribute values, or computed
color -- everything oxaria needs to read DOM/ARIA attributes and
resolve text/background color. The underlying C++ engine already has all of
this internally (`element::get_tagName()`, `element::get_attr()` /
`dump_get_attrs()`, `css_properties::get_color()`,
`element::get_background()`); it just wasn't wired through the C wrapper.

Added to `litehtml_c/litehtml_c.h` and `.cpp`:

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
  `litehtml/include/litehtml/html_tag.h` (the engine itself, not just
  the wrapper) got a one-line public getter, `attrs()`; the wrapper reaches it
  with a `dynamic_cast<html_tag*>` (text and comment nodes aren't `html_tag`s
  and yield nothing). litehtml also writes one attribute of its own there,
  `list_index` on list items during layout, which oxaria filters out.
- `lh_element_get_color` -- computed CSS `color`, resolved through inheritance.
- `lh_element_get_background_color` -- the element's own declared
  `background-color` (backgrounds don't inherit in CSS, so callers wanting
  the effective visual background walk up via `lh_element_parent` themselves
  until a non-transparent color or the root is reached).

Matching declarations were hand-added to the pre-generated
bindings (now `src/litehtml/sys.rs`; upstream's default build skips `bindgen`
and uses this file, so no libclang is needed to build), and matching safe
methods (`tag_name`, `attr`, `attrs`, `color`, `background_color`) were added
to the `Element` impl (now `src/litehtml/engine.rs`).

`Element<'a>` was given `#[derive(Clone, Copy)]` -- it only ever wraps a
borrowed pointer (like any other `&_`), so duplicating it is exactly as safe
as copying a reference; all unsafety stays contained within the methods that
dereference it. This lets oxaria walk up an element's ancestor chain
(`while let Some(parent) = el.parent() { el = parent; }`) without threading
lifetimes through a recursive helper.

`build.rs` was fixed for the release builds (Windows, and Linux
cross-compiled with `cross`), which upstream's never targeted:

- It picks the C++ standard library from `CARGO_CFG_TARGET_*`
  (`link_cpp_stdlib`). Upstream used `cfg!(target_os = "macos")`, which in a
  build script reads the host, and linked `stdc++` everywhere else, which
  doesn't exist on MSVC.
- On MSVC it adds what litehtml's own CMakeLists does: gumbo's `visualc/include`
  (a `<strings.h>` shim, MSVC has none) and `/utf-8` for the C++ sources.

`lh_document_add_stylesheet` (`litehtml_c.cpp`) passes the CSS to
`css::parse_css_stylesheet` as a `std::string` instead of the raw
`const char*`. That template is defined only in litehtml's `stylesheet.cpp`,
which instantiates it for `string` and `css_token_vector`, so upstream's call
referenced an undefined `parse_css_stylesheet<const char*>`. GNU ld never
noticed (the function is garbage-collected when unused), but MSVC's linker
fails with `LNK2019`.

## Known limitation carried into oxaria

litehtml has no `:focus`/`:hover`/`:active` state-simulation API, so nothing
here can compare an element's focused vs. unfocused computed style. The
focus-outline rule (G195) keeps its own `<style>`-text-scanning heuristic
rather than relying on the engine for that specific check.
