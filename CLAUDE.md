# wcag-checker

Rust CLI that fetches a URL or reads a local HTML file, renders it through an embedded
rendering engine, and checks the result against a curated set of WCAG 2.2, WAI-ARIA 1.2, ARIA
in HTML and ACT rules in parallel,
exiting 0/1/2 for clean/error/warning-only results.

## Usage

See [README.md](./README.md) for CLI usage, exit codes, and the architecture diagram. This file
is about *why* things are built the way they are, for whoever touches this code next.

## Where standards/wcag2.2-rules.md comes from

`standards/wcag2.2-rules.md`  was written by grounding every rule ID, title, and DO/DON'T example in the actual WCAG 2.2
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
and one `Arc<RenderedPage>` can be cloned into every parallel `spawn_blocking` rule task directly --
no per-task re-rendering, no lifetime gymnastics. This is a real simplification over the
`scraper`-based version, which had to re-parse the HTML string inside every spawned task to work
around `Html` not being `Sync`.

### Why contrast/target-size are "spectrum" (warning) rules and everything else is an error

This was an explicit, fixed design split, not a per-finding judgment call: rules that measure a
continuous quantity against a configurable WCAG threshold (contrast ratio, pointer target size in
px) produce `Severity::Warning` and respect a `--*-threshold` CLI flag; every other rule is a
binary pass/fail and always produces `Severity::Error`. Don't add more CLI thresholds or promote
another rule to "spectrum" status without confirming that's actually wanted -- it changes exit
code semantics (`0` clean / `1` any error / `2` warnings only). ACT's afw4f7 is the same
spectrum rule as G18 and reads the same two contrast flags; its Level AAA sibling 09o5cg was
dropped because those flags only express Level AA, so its warnings could never be cleared.

## Rule scope: what's implemented and why the rest isn't

`standards/wcag2.2-rules.md` lists ~48 rules across 15 categories. Only the ones genuinely verifiable from
a single static render are implemented (see `rule_checks()` in `src/rules/wcag22/checks.rs` for
the exact list). Categories deliberately **not** implemented, and why:

- **Timing/motion, dragging, redundant entry, accessible authentication, consistent help**
  (session timeouts, drag-alternative interactions, multi-step form memory, login flows) --
  these require understanding a multi-page *process* and user intent, not a page snapshot. No
  real accessibility checker (axe, Lighthouse, WAVE) auto-verifies these either; they're
  flagged for manual review industry-wide. The exceptions are the parts written into the
  markup itself: a timed `<meta http-equiv="refresh">` (F40/F41) and an `onpaste` handler
  cancelling paste into a password/code field (AUT001).
- **Inline event handlers are the only script the rules see** (F42, F54, F55, F59, AUT001):
  they read `on*="..."` attributes, never `addEventListener` calls, which would need JS
  execution.
- **Failures that need interaction are listed in `standards/wcag2.2-rules.md` but not implemented**:
  F95 (hover content), F97 (orientation), F99 (single-key shortcuts), F101 (down-event),
  F103 (status messages), F105 (path gestures), F108 (dragging), F109 (split code fields),
  F110 (focus obscured). F94 (viewport-unit font sizes) is also left out: litehtml resolves
  `vw` to px, and a stylesheet scan can't tell whether media queries rescue the text.
- **Keyboard trap / keyboard operability, ARIA state-toggle verification** -- would need real
  event dispatch and JS execution, which litehtml doesn't provide (see above).
- **G195 (focus outline) is implemented, but only as a heuristic**: it scans raw `<style>` tag
  text (captured directly from the source HTML string via `page::render::extract_stylesheets`,
  independent of the render tree) for `:focus` selectors that set `outline: none` without an
  alternative indicator. This is the one rule that does *not* use `RenderedPage`'s computed
  style, because there is nothing to compute -- litehtml never enters a focused state.

### WAI-ARIA 1.2, ARIA in HTML and ACT

The same criterion applies. A rule with a `Prevailing rule:` line in its standard's file lost a
conflict in `standards/overlap.md` and is not implemented, since reporting both sides would give
contradictory advice. Everything else that overlaps *is* reported by every standard under its own
ID (explicitly requested: e.g. a missing `aria-describedby` target is ARIA1 and ARIA-IDREF001, an
invalid `aria-expanded` value is ARIA-VAL001 and ACT 6a7281). These duplicates are intended; don't
deduplicate across standards.

**WAI-ARIA 1.2** not implemented:
- Losers: ROLE001, ATTR002, ATTR006, VAL004, IDREF004, NAME001, STATE002, STATE003, USAGE004.
- Need JS, interaction or human judgement: ROLE004, WIDGET004, WIDGET005, FOCUS002-005,
  FOCUS007, STATE001, STATE004, STATE005, LIVE001 (not reported, per overlap C12), LIVE002,
  USAGE001.
- IDREF005: whether a tooltip is displayed depends on CSS/JS state litehtml doesn't show.
- Partial, checking only their reliable part: ATTR009, VAL002, VAL005, USAGE002, USAGE003,
  WIDGET006, WIDGET007, FOCUS001, FOCUS006.
- ATTR001 accepts the WAI-ARIA 1.3-only names in `aria_spec::ARIA_1_3_ATTRIBUTES` (overlap S7):
  browsers already support some, so rejecting them would be a false positive.

**ARIA in HTML** not implemented: losers HTMLARIA001, 005, 008, 012; `Automation: none`
HTMLARIA006 (needs to know the page's audience) and 018 (the parser repairs the nesting before
any DOM exists).

**ACT** not implemented:
- Loser: 4e8ab6.
- Human judgement: c4a8a4, qt1vmo, 0va7u6.
- Rendering litehtml can't give: oj04fd (focus state), akn7bn (nested iframe documents), 0ssw9k
  (computed overflow), 78fd32 (real text wrapping), 6cfa84 (no computed `display`/`visibility`,
  so menus hidden by CSS would be false positives).
- 09o5cg: see the spectrum-rule section above.
- b33eff is partial: only quarter turns inside `orientation` media queries.

ACT has its own accessible-name computation (`act/name.rs`), more complete than
`ElementRef::accessible_name`, because its rules are defined against the full accname algorithm.
`act/language_subtags.rs` is the IANA language subtag registry (File-Date 2025-08-25), taken from
the mattcg/language-subtag-registry mirror because iana.org was blocked; refresh it from IANA
when possible. `aria_spec/` holds the WAI-ARIA 1.2 and ARIA in HTML tables all three standards
share; it keeps only the columns some rule reads, so an implemented loser may need data re-added.

If you're asked to add a new rule, check `standards/wcag2.2-rules.md` first for its ID/wording, then check
whether it's actually derivable from `RenderedPage` (DOM structure/attributes + computed
color/background/font metrics + layout box) before starting -- if it needs JS, multi-page state,
or focus/hover simulation, it belongs in the "not implemented" list above, not a half-working
heuristic bolted onto `RenderedPage`.

## Architecture

See [README.md](./README.md#architecture) for the module tree. Notes beyond what's there:

- **One module directory per standard under `src/rules/`** (`wcag22/`, `aria12/`, `html_aria/`,
  `act/`, plus their shared `aria_spec/` data), each exposing `rule_checks()` and `reference_url()`, so rules of different standards
  can be added without touching shared files. Rule functions don't know their standard: the
  registry (`rules/registry.rs`) pairs each list with its `Standard` and stamps it on every
  finding. Failures of the checker itself (`FETCH`, `HTTP`, `SCAN`, ...) have no standard. In
  `wai-aria-1.2-rules.md`, `html-aria-rules.md` and `act-rules.md`, a rule with a
  `Prevailing rule:` line is intentionally not implemented.

- **`page/render.rs`** is the only file that touches the `litehtml` crate directly. Everything
  else (`page/model.rs`, all of `rules/`) only ever sees `RenderedPage`/`RenderedElement`/
  `ElementRef` -- plain data, no engine types leak out. Keep it that way; it's what makes the
  data model renderer-agnostic and the unit tests engine-free.
- **Every source attribute reaches `RenderedElement::attrs`**: `render.rs` enumerates each
  element's attributes through a patched accessor (`Element::attrs()`, see PATCHES.md), names
  lowercased. Before that, litehtml only allowed lookup by name and a fixed
  `ATTRIBUTES_OF_INTEREST` allowlist was probed, so unknown names (`aria-labeledby`) were
  invisible. The one engine-set attribute, `list_index` on `li`, is filtered out.
- **`src/rules/aria_spec/`** holds the WAI-ARIA 1.2 and ARIA in HTML data shared by the
  `aria12`, `html_aria` and `act` rules, and the helpers built on it (`effective_role`,
  `implicit_role`, `allowed_roles`/`allowed_aria`, `is_focusable`, `is_hidden`, `aria_value`,
  ...). `role_data.rs`/`attribute_data.rs` were extracted by script from w3c/aria's
  `index.html` (`2023-06_REC` branch, commit `66caad8`), inheritance resolved like the spec's
  own `common/script/aria.js`, and checked against the tables in
  `standards/wai-aria-1.2-rules.md`; `html.rs` follows the document conformance table of
  w3c/html-aria at `e277aa3` (`REC-html-aria-20260811`). Re-check against those repos (cloned)
  before changing any entry. Rules use this module rather than their own copies; the `wcag22`
  rules predate it and keep theirs.
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
  a false positive. **Targets with unsizable content go further** (`has_unsized_content`): litehtml
  has no intrinsic size for form controls, never loads images, and ignores `<svg>`'s
  `width`/`height` (all verified: they render 0×0). So for an `<input>`, or a text-less target
  wrapping `img`/`svg`/`picture`/`video`/`canvas`, a 0 on *either* axis means "not measured" and
  only the other axis is used. Without this, google.com's submit buttons (CSS height only),
  gitlab.com's logo links (CSS width only) and docker.com's icon buttons were reported as 0px.

### Full-site scan (`src/site/`)

- **Everything HTTP lives behind `request_helper::request()`** (explicitly requested): rate
  limiting, retries, redirects, and content-type classification. The client is built with
  `redirect::Policy::none()` and redirects are followed by hand inside `request()`, one hop at a
  time, so the same-host check, the hop limit, and rate-limit waits between hops all live in
  that one function instead of being split into a separate `redirect::Policy::custom` closure.
  Off-domain redirects are skipped silently, not reported.
- **Every request goes through `page::client_builder()`** -- a browser-style `Accept`, a
  `wcag-checker/<version>` `User-Agent` and a timeout -- from both single-page (`fetch.rs`) and
  full-site (`request_helper.rs`) mode. reqwest's defaults (`Accept: */*`, no `User-Agent`) got
  github.com/marketplace answering `400` (content negotiation picked JSON) and crates.io
  answering `403` (it requires a User-Agent). Keep the `*/*;q=0.8` fallback -- without it,
  API/PDF URLs may answer `406` and be reported as errors instead of being skipped.
- **Rate limiting is shared**: a single `RwLock<Instant>` in the shared `RequestContext` means a
  `429` seen by one task pauses every task, not just the one that got it.
- **Logging**: one `Fetching <url>` line per requested URL, then an indented
  `<from> redirects to <to>` line per hop. An earlier version logged `Fetching` per hop, which
  made redirect chains look like requests that never produced a result.
- **Visit key is the URL path only** (no query, no fragment), since the crawl never leaves
  one host. This is what prevents infinite crawls through pagination/facet query strings.
  `/a` and `/a/` are deliberately distinct.
- **Same domain = exact host match** (`www.example.com` ≠ `example.com`), port ignored.
- **The "already fetched" cache lives in `RequestContext` and is checked by `request()`
  itself, right before every download -- including each redirect hop** (explicitly requested:
  a path must never be downloaded twice). The claim is a check-and-insert under one lock so
  racing tasks can't both win. The crawler's `queued` set and `has_fetched` check only stop the
  same link from being scheduled twice (ubuntu.com links `/navigation` 6 times per page); they
  are not what guarantees a path is downloaded once.
- **`--full-site-scan-max-pages` counts only processed HTML pages** (explicitly requested) --
  not API/PDF/other responses, HTTP errors, or already-fetched skips.
- **The crawl is breadth-first, one level at a time, and that's what makes it deterministic.**
  An earlier version enqueued links as soon as each page finished, so under a page budget
  *which* pages got checked depended on download speed (two ubuntu.com runs checked different
  pages). Now a level's links are ordered by the discovering page's position in its level, then
  document order. The cost is a sync point between levels.
- **In-flight pages never exceed the budget's remaining slots** (`Crawl::capacity`): every
  in-flight page might be HTML, so each already holds a slot. This is what prevents wasted
  downloads near the limit (an earlier version fetched 12 pages to check 5 on github.com) --
  near the limit, fetches become sequential.
- **The start URL may redirect to any host** (`RedirectScope::AnyHost`), and the host it lands
  on becomes the crawl's host. Every later request uses `RedirectScope::SameHost`. Before this,
  `gitlab.com` -> `about.gitlab.com` checked nothing and exited `0`.
- **A crawl that checks no page and has no other finding reports a `SCAN` error**, so a non-HTML
  start URL can't read as a clean pass.
- HTTP/network/render failures during a crawl become `Finding`s (`HTTP`, `FETCH`, `RENDER`)
  tagged with the page path, rather than aborting the run as they do in single-page mode.
- Tests use `wiremock`, addressing the main server as `localhost` and a second "external"
  server as `127.0.0.1`, so they are genuinely different hosts.

### Reporters (`src/reporters/`)

- **Every output goes through one `Report` model** (`model.rs`): findings plus a `Location` and
  a fingerprint. Each CI format is a `Reporter` (a trait, explicitly requested, despite the
  "fn pointers" convention below) rendering a whole `Report` to a `String`. Whole-buffer
  rendering is what makes the parallel writes in `output.rs` safe: each report is one write, so
  nothing interleaves mid-line on stdout.
- **Only formats CI platforms read natively** (GitLab Code Quality, GitHub job summary,
  Jenkins Warnings NG, JUnit). A custom JSON format and SARIF were explicitly rejected.
- **GitHub gets a Markdown job summary, not annotations** (explicitly requested): workflow
  command annotations are capped at 10 errors + 10 warnings per step, so most findings were
  invisible. `--report-github` defaults to `$GITHUB_STEP_SUMMARY` (clap `env`); GitLab and
  Jenkins have no built-in report-path variable (checked: GitLab's predefined variables, a real
  Jenkins pipeline's `env`). The summary is *appended* (`Reporter::appends`), as GitHub documents,
  kept under GitHub's 1 MiB step limit, and an empty value (`GITHUB_STEP_SUMMARY=`) disables it.
  Tests must clear that variable, or running them on GitHub Actions writes to the job summary.
- **Stdout output is unchanged by `--report-*`**; only `-q/--quiet` removes it (findings,
  summary, and `tracing` logs, which go to stdout). A report sent to `-` still prints under
  `--quiet`, since it was asked for explicitly.
- **Destination conflicts are checked before the check runs** (explicitly requested): two
  reports on one file (compared after lexical `..` normalization and canonicalizing the parent,
  so different spellings collide), two reports on stdout, or a report overwriting the input
  file all exit `1` immediately.
- **No line numbers**: litehtml exposes no source positions, so every issue is on line `1`
  (`model::LINE`). GitLab requires a line; don't drop the field.
- **Fingerprints must be unique and stable**: GitLab merges issues sharing a fingerprint and
  compares them across pipelines. They're FNV-1a (std's `DefaultHasher` isn't stable across
  Rust releases) of standard/rule/path/page/message plus an occurrence counter, so two identical
  findings (two unlabeled checkboxes) stay two issues.
- **Consumers silently merge issues they consider identical, each by its own key** (found by
  running their real parsers, not from the docs): Warnings NG's equality ignores `fingerprint`,
  so `jenkins.rs` also puts it in `additionalProperties`; GitLab's JUnit parser keys test cases by
  suite + classname + name, so `junit.rs` names each case `<standard> <rule>: <message>` with a ` (n)`
  suffix on repeats. Without these, three unlabeled checkboxes showed up as one issue.
- **The JUnit report is not for Warnings NG** (verified in a real Jenkins): its JUnit parser
  derives locations from Java stack traces and ignores passing/skipped tests, so warnings are
  lost and URL locations come out garbled whatever we write. `--report-jenkins` is the complete
  Warnings NG route; JUnit is complete for Jenkins' `junit` step and GitLab.
- **Findings name their element** (`Finding::at` -> `ElementRef::selector()`), because a page
  can have 138 identical `<button> target size is 19px` warnings. The path is structural
  (`tag:nth-of-type(n)`, counting same-tag siblings), skips `html` and litehtml's tag-less
  anonymous boxes, and stops at the nearest element whose id is unique *and* CSS-safe (a
  duplicate id, which IDS001 reports, would select two elements). It was verified to select exactly
  its element in an HTML5 parse (html5lib + soupsieve) of github.com pages and the fixtures.
  Reporters append it to the message (`(at ...)`), since no CI format has a field for it, and
  that makes fingerprints independent of finding order. Rules about several elements (IDS001,
  LNK001), the page (H42, H57, G1 without a link) or a stylesheet (G195) leave it `None`.
- **Findings carry a rustc-style `help`** (`Finding::help`), set by the rule itself rather than
  looked up by rule ID: H30 and G1 each report two different problems under one ID, and the
  most useful hints need the finding's context (G18's nearest passing color, TGT001's missing
  padding, G141's expected level). Each standard's `checks.rs` has an `every_rule_fires_with_help`
  test (`registry::tests::assert_every_rule_has_help`) that fails if a rule ships without one.
  The `note: see <url>` link comes from `Standard::reference_url`: for WCAG, from the rule ID's
  prefix (URL shapes taken from the w3c/wcag repo's own cross-links); for WAI-ARIA and ARIA in
  HTML, from the `ANCHORS` table in their `reference.rs`; for ACT, from the ID itself.
- **Help stays out of `Issue.message`**, which the fingerprint and JUnit test names are built
  from: rewording a hint must not make GitLab see every issue as fixed-and-new. Formats with a
  single text field get `Issue::full_text()`; GitHub puts help in the message cell and links the
  rule ID; Warnings NG gets it as HTML in `description` (verified in a real Jenkins: shown under
  the bold message in the issue's details row, link clickable).
- **GitLab's description is Markdown in one view and plain text in another** (found by running
  GitLab's frontend rendering, `marked` + its strict DOMPurify config, not from the docs): the
  pipeline Code Quality tab passes raw HTML through, so a quoted `<video>` erased its whole
  message and `<h1>` broke the layout, while the MR widget escapes it. `gitlab.rs` wraps every
  `<...>` in backticks (a code span in one, readable in the other) and ends lines with two spaces
  (a line break in one, invisible in the other).
- **`Finding.page` is the page's final URL, not its path** (query and fragment dropped), so a
  start URL that redirects to another host (`www.openai.com` -> `openai.com`) locates findings
  on the host that actually served them. The text output still shows only the path.
- **Logs are only coloured on a terminal** (`logging.rs`): CI logs and redirected files got
  raw ANSI codes, which Jenkins shows as garbage without its AnsiColor plugin.
- **Fatal errors become issues** (`CheckerError::rule_id()` -> `INPUT`/`FETCH`/`RENDER`), not
  log lines, so CI gets a report explaining the failure instead of a missing artifact.

## Gotchas

- **Raw string literals containing `href="#..."` fragments will silently truncate.**
  `r#"<a href="#main">...</a>"#` breaks because the parser treats the first `"#` it finds --
  which is right inside `="#main"` -- as the closing delimiter, ending the string early with a
  confusing cascade of unrelated parse errors afterward. Use `r##"..."##` (or more hashes) for
  any test fixture HTML containing a `#`-fragment `href`. This has already bitten this codebase
  once (`src/rules/wcag22/navigation.rs`, `src/rules/wcag22/checks.rs`); watch for it in new fixtures.
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
