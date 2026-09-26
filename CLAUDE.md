# wcag-checker

Rust CLI that fetches a URL or reads a local HTML file, renders it through an embedded
rendering engine, and checks the result against a curated set of WCAG 2.2 rules in parallel,
exiting 0/1/2 for clean/error/warning-only results.

## Usage

See [README.md](./README.md) for CLI usage, exit codes, and the architecture diagram. This file
is about *why* things are built the way they are, for whoever touches this code next.

## Where wcag2.2-rules.md comes from

`wcag2.2-rules.md`  was written by grounding every rule ID, title, and DO/DON'T example in the actual WCAG 2.2
Techniques/Failures/Guidelines HTML under the <https://github.com/w3c/wcag> repo (`techniques/html`, `techniques/aria`,
`techniques/general`, `techniques/failures`, `guidelines/sc/22` for the six success criteria new
in 2.2).

If you're adding a new rule or second-guessing an existing one's wording, that parent
repo is the source of truth to re-check against -- don't invent rule text from general WCAG
knowledge alone. Clone the repo instead of using the web API if you ever need to interract with it.

Only a subset of the rules listed there are actually implemented (see "Rule scope" below).

## Design decisions

### Why an embedded rendering engine at all

The first version of this checker parsed HTML with `scraper` and read colors/sizes from inline
`style=""` attributes via regex-ish string parsing. That's a real accessibility checker's
fundamental limitation: no CSS cascade, no real layout, so contrast and target-size checks only
caught issues declared inline. Rendering the page for real (computed styles, real layout boxes)
was the explicit ask that led to everything under `page/` and `vendor/`.

### Why litehtml, not a full browser

The user's constraint was explicit: no browser download at runtime, and "something lighter" than
a full browser if a real embed wasn't practical. Options considered (see conversation history if
you want the full comparison): embedding Chromium via CDP (real JS + native AX tree, but a
150-300MB per-platform binary and heavy build/CI complexity -- rejected as too heavy), a pure-Rust
`blitz` stack (viable but young/unstable at evaluation time), hand-building a pipeline from
`scraper` + `cssparser` + `taffy` (100% control, but a large from-scratch engineering effort), and
litehtml (real CSS cascade + real layout, no JS, and its Rust bindings already vendor litehtml's
own C++ source and compile it via the `cc` crate -- genuinely no runtime download, no separate
browser process). litehtml won on the effort/capability tradeoff.

**Consequence:** there is no JavaScript execution and no `:focus`/`:hover`/`:active` state
simulation. Rules that would need either (keyboard-trap testing, ARIA-state-toggle verification,
comparing an element's focused vs. unfocused style) are out of scope with this engine -- see "Rule
scope" below for how that's handled today, and don't assume a future engine swap is free (it
would touch `page/render.rs` and every rule that depends on `RenderedElement` fields, though the
`RenderedPage` data model itself was designed to be renderer-agnostic).

### Why the vendored crates are patched

The upstream `litehtml`/`litehtml-sys` crates (`franzos/litehtml-rs`) expose layout data
(bounding box, font size, line height) but not tag name, attribute values, or computed color --
exactly what every DOM/ARIA-based rule needs. The underlying C++ engine already has all of this
(`element::get_tagName()`, `element::get_attr()`, `css_properties::get_color()`,
`element::get_background()`); it just wasn't wired through the thin C wrapper. Rather than fork
the whole project, `vendor/litehtml-sys` and `vendor/litehtml` are local copies with ~4 small
C-wrapper functions added, following the exact pattern of the existing wrapper functions. Full
details, including a documented dead end (`dump_get_attrs()` looked like an attribute enumerator
but actually dumps computed CSS properties for the engine's own debug printer -- verified
empirically, not just by reading the header), are in **[vendor/PATCHES.md](./vendor/PATCHES.md)**.
Read that file before touching anything under `vendor/`, and update it if you patch further.

If litehtml-rs upstream ever adds these accessors itself, `vendor/` could be dropped in favor of
a normal crates.io dependency -- check PATCHES.md against the upstream changelog before doing
that migration.

### Why one `RenderedPage`, not a two-tier rule system

An earlier plan split rules into an `HtmlRule` trait (static DOM/attribute checks) and a
`BrowserRule` trait (computed style/layout checks), gated behind an opt-in CLI flag so existing
tests/behavior wouldn't change by default. The user explicitly rejected both parts of that: no
CLI flag ("the user should not have to choose"), and no trait split -- every rule takes
`&RenderedPage`, full stop, and rendering is unconditional. This is why `rules::RuleCheck` is a
plain `fn(&RenderedPage, &CheckOptions) -> Vec<Finding>` type alias rather than a trait object --
with only one rule shape, a trait would add indirection without buying anything.

### Why `RenderedPage` is an arena, not borrowed references

`scraper::Html` and `litehtml::Element` are both non-`Sync` (the latter is explicitly `!Send +
!Sync`, wrapping a raw C++ pointer). `RenderedPage`/`RenderedElement` are plain owned data
(`String`, `f32`, `Vec<usize>` parent/child indices) precisely so they're trivially `Send + Sync`
and one `Arc<RenderedPage>` can be cloned into every parallel `tokio::spawn` rule task directly --
no per-task re-rendering, no lifetime gymnastics. This is a real simplification over the
`scraper`-based version, which had to re-parse the HTML string inside every spawned task to work
around `Html` not being `Sync`.

### Why contrast/target-size are "spectrum" (warning) rules and everything else is an error

This was an explicit, fixed design split, not a per-finding judgment call: rules that measure a
continuous quantity against a configurable WCAG threshold (contrast ratio, pointer target size in
px) produce `Severity::Warning` and respect a `--*-threshold` CLI flag; every other rule is a
binary pass/fail and always produces `Severity::Error`. Don't add more CLI thresholds or promote
another rule to "spectrum" status without confirming that's actually wanted -- it changes exit
code semantics (`0` clean / `1` any error / `2` warnings only).

## Rule scope: what's implemented and why the rest isn't

`wcag2.2-rules.md` lists ~48 rules across 15 categories. Only the ones genuinely verifiable from
a single static render are implemented (see `src/rules/mod.rs`'s `all_rule_checks()` for the
exact list). Categories deliberately **not** implemented, and why:

- **Timing/motion, dragging, redundant entry, accessible authentication, consistent help**
  (session timeouts, drag-alternative interactions, multi-step form memory, login flows) --
  these require understanding a multi-page *process* and user intent, not a page snapshot. No
  real accessibility checker (axe, Lighthouse, WAVE) auto-verifies these either; they're
  flagged for manual review industry-wide.
- **Keyboard trap / keyboard operability, ARIA state-toggle verification** -- would need real
  event dispatch and JS execution, which litehtml doesn't provide (see above).
- **G195 (focus outline) is implemented, but only as a heuristic**: it scans raw `<style>` tag
  text (captured directly from the source HTML string via `page::render::extract_stylesheets`,
  independent of the render tree) for `:focus` selectors that set `outline: none` without an
  alternative indicator. This is the one rule that does *not* use `RenderedPage`'s computed
  style, because there is nothing to compute -- litehtml never enters a focused state.

If you're asked to add a new rule, check `wcag2.2-rules.md` first for its ID/wording, then check
whether it's actually derivable from `RenderedPage` (DOM structure/attributes + computed
color/background/font metrics + layout box) before starting -- if it needs JS, multi-page state,
or focus/hover simulation, it belongs in the "not implemented" list above, not a half-working
heuristic bolted onto `RenderedPage`.

## Architecture

See [README.md](./README.md#architecture) for the module tree. Notes beyond what's there:

- **`page/render.rs`** is the only file that touches the `litehtml` crate directly. Everything
  else (`page/model.rs`, all of `rules/`) only ever sees `RenderedPage`/`RenderedElement`/
  `ElementRef` -- plain data, no engine types leak out. Keep it that way; it's what makes the
  data model renderer-agnostic and the unit tests engine-free.
- **`ATTRIBUTES_OF_INTEREST`** in `render.rs` is a fixed allowlist of HTML attribute names probed
  per element (since litehtml only exposes single-attribute lookup by name, not enumeration --
  see PATCHES.md). If you add a rule that needs an attribute not already in that list, add it
  there or the attribute will silently read as absent.
- **Font weight** isn't exposed as a queryable computed-style property by litehtml at all.
  It's captured a different way: litehtml calls back into `Container::create_font` once per
  distinct font description (including weight) and hands back a `FontHandle`; `render.rs` records
  `FontHandle -> weight` in a map and looks it up per element via `Element::font()`. If you need
  another font-derived property later, this is the pattern to extend.
- **`Rc<RefCell<HashMap<...>>>` for the font map in `render.rs`** isn't incidental complexity --
  `litehtml::Document<'a>` holds an exclusive `&'a mut` borrow of the `DocumentContainer` for its
  entire lifetime (needed for callbacks during `render()`), so the tree-walk afterwards can't also
  borrow `container.fonts` directly; the font map has to live behind a separate `Rc` handle kept
  outside `container` so it can be read once the mutable borrow is otherwise idle.
- **Text measurement is approximate** (`text_width` in `render.rs` uses an average
  character-width heuristic, not real glyph shaping). This is intentional -- no rule here depends
  on pixel-accurate line wrapping, only on computed color and the bounding boxes of elements whose
  size is set by CSS (buttons, inputs, table cells, ...) rather than by unconstrained text flow.
  If a future rule needs real text measurement (e.g. a reflow/zoom check), this will need
  revisiting with a real shaping library.
- **`target_size.rs`'s `bbox.width == 0.0 && bbox.height == 0.0` skip is deliberate**, not a bug:
  it's the sentinel for "not laid out" (e.g. `display: none`, or -- in hand-built
  `page_from_html` test fixtures with no declared size -- simply never measured). A genuinely
  laid-out 0×0 interactive element would be a real finding; a not-measured one shouldn't produce
  a false positive.

### Full-site scan (`src/site/`)

- **Everything HTTP lives behind `request_helper::request()`** (explicitly requested): rate
  limiting, retries, redirects, and content-type classification. The client is built with
  `redirect::Policy::none()` and redirects are followed by hand inside `request()`, one hop at a
  time, so the same-host check, the hop limit, and rate-limit waits between hops all live in
  that one function instead of being split into a separate `redirect::Policy::custom` closure.
  Off-domain redirects are skipped silently, not reported.
- **Every request sends `page::request_headers()`** -- a browser-style `Accept` and a
  `wcag-checker/<version>` `User-Agent` -- from both single-page (`fetch.rs`) and full-site
  (`request_helper.rs`) mode. reqwest's defaults (`Accept: */*`, no `User-Agent`) got
  github.com/marketplace answering `400` (content negotiation picked JSON) and crates.io
  answering `403` (it requires a User-Agent). Keep the `*/*;q=0.8` fallback -- without it,
  API/PDF URLs may answer `406` and be reported as errors instead of being skipped.
- **Rate limiting is shared**: a single `Arc<RwLock<Instant>>` in `RequestContext` means a
  `429` seen by one task pauses every task, not just the one that got it.
- **Visit key is the URL path only** (no query, no fragment), since the crawl never leaves
  one host. This is what prevents infinite crawls through pagination/facet query strings.
  `/a` and `/a/` are deliberately distinct.
- **Same domain = exact host match** (`www.example.com` ≠ `example.com`), port ignored.
- **The "already fetched" cache lives in `RequestContext` and is checked by `request()`
  itself, right before every download -- including each redirect hop** (explicitly requested:
  a path must never be downloaded twice). The claim is a check-and-insert under one lock so
  racing tasks can't both win. The orchestrator's `has_fetched` check before spawning is only
  an optimisation to avoid spawning no-op tasks; it is not what guarantees uniqueness.
- **`--full-site-scan-max-pages` counts only processed HTML pages** (explicitly requested) --
  not API/PDF/other responses, HTTP errors, or already-fetched skips. `PageBudget` is a
  semaphore with `max_pages` permits, reserved *before* each download: an HTML response
  `forget()`s its permit (slot consumed), anything else drops it (slot handed back to a waiting
  task). This keeps in-flight downloads <= remaining slots, so the budget can't cause wasted
  downloads (an earlier reserve-after-download version fetched 12 pages to check 5 on
  github.com). The semaphore is closed when the last slot is consumed -- that's what wakes
  tasks still waiting in `reserve()`; without it they'd wait forever and the crawl would hang.
  Budget is acquired before the concurrency permit, so waiting tasks don't hog concurrency.
- HTTP/network/render failures during a crawl become `Finding`s (`HTTP`, `FETCH`, `RENDER`)
  tagged with the page path, rather than aborting the run as they do in single-page mode.
- Tests use `wiremock`, addressing the main server as `localhost` and a second "external"
  server as `127.0.0.1`, so they are genuinely different hosts.

## Gotchas

- **Raw string literals containing `href="#..."` fragments will silently truncate.**
  `r#"<a href="#main">...</a>"#` breaks because the parser treats the first `"#` it finds --
  which is right inside `="#main"` -- as the closing delimiter, ending the string early with a
  confusing cascade of unrelated parse errors afterward. Use `r##"..."##` (or more hashes) for
  any test fixture HTML containing a `#`-fragment `href`. This has already bitten this codebase
  once (`src/rules/navigation.rs`, `src/rules/mod.rs`); watch for it in new fixtures.
- **`litehtml::Element<'a>` is `Clone + Copy`** (patched -- see PATCHES.md) specifically so
  ancestor-walking loops (`while let Some(parent) = el.parent() { el = parent }`) don't need to
  thread ownership through a recursive helper. If you find yourself needing another
  ergonomics-only trait on `Element`, adding it to the vendored copy is the established pattern --
  just document it in PATCHES.md.
- **`master.css`** (`src/page/master.css`) is litehtml's own default user-agent stylesheet,
  extracted by hand from `vendor/litehtml-sys/vendor/litehtml/include/litehtml/master_css.h`
  (stripping the C++ raw-string wrapper). It's what gives elements realistic default styling
  (headings bold/larger, links blue+underlined, etc.) without it, computed styles for
  un-styled pages would be far less representative of a real browser. If litehtml is ever
  upgraded, re-extract this file from the new `master_css.h`.

## Coding conventions

- Small, single-purpose functions over large ones; prefer `match` over `if`/`else` chains where
  there's more than a binary branch.
- `mod.rs` files contain only `mod`/`use` declarations, never logic.
- No comments explaining *what* code does (names should do that) -- only comments explaining a
  non-obvious *why* (a hidden constraint, a workaround, a subtle invariant). Several of the
  "Gotchas" above are exactly the kind of thing worth a one-line comment at the call site, not
  just in this file.
- `thiserror` for the error enum (`CheckerError`), `tracing`/`tracing-subscriber` for logging,
  `clap` derive for the CLI, plain `fn` pointers over trait objects when every implementor has
  the same signature.

## Tests

- Unit tests build `RenderedPage` fixtures from plain HTML strings via
  `page::testutil::page_from_html` (`scraper`/`html5ever`, a **dev-dependency only** -- production
  code never uses `scraper`). This parses real markup for structure/attributes/text for free, and
  does a simple *non-cascading* read of each element's own inline `style=""` for
  `color`/`background-color`/`font-size`/`font-weight`/`width`/`height` (no class selectors, no
  inheritance simulation) for the handful of rules that need it. This keeps unit tests fast and
  independent of the rendering engine -- avoid reaching for the real `page::render()` in a unit
  test unless you're specifically testing the render pipeline itself (`src/page/render.rs`'s own
  tests do use it, deliberately).
- Integration tests (`tests/integration.rs`) run the compiled binary against real fixtures under
  `tests/assets/*.html` through the actual embedded engine end-to-end. If you change rendering
  behavior, expect these to need re-verification against real computed output, not just updated
  assertions -- the real engine has already caught at least one genuine bug this way (a button in
  `tests/assets/clean.html` rendering under the 24px target size with no explicit sizing).

Run: `cargo test`

## Checklist

Before returning to the user, make sure the code is formatted and linter and tests pass:

- `cargo clippy --fix --allow-dirty --` → `cargo clippy --fix --tests --allow-dirty --` to
  auto-fix issues → fix remaining issues → repeat until no issue found
- `cargo test`
- `cargo fmt`
