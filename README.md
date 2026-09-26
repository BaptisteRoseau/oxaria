# wcag-checker

Check a web page against a curated set of WCAG 2.2 rules — fetch a URL or read a local HTML file, render it through an embedded rendering engine, and report errors/warnings.

Built in Rust. No headless browser process, no runtime download — the rendering engine is vendored and statically compiled directly into the binary.

## Overview

`wcag-checker` fetches (or reads) a page's raw HTML, renders it through an embedded [litehtml](https://github.com/litehtml/litehtml) engine to get real computed styles and layout, and runs every rule against the result in parallel via Tokio tasks.

- **Real rendering, not string parsing** — contrast and target-size checks use the engine's actual computed color, background, and layout box, so they catch issues regardless of how color or size was set (CSS class, cascade, padding around content), not just inline styles.
- **No runtime download** — the rendering engine's C++ source is vendored and statically linked into the binary at build time; nothing is fetched when the tool runs.
- **Parallel by design** — every rule runs as its own `tokio::task` against a shared, immutable snapshot of the rendered page.
- **Two exit-code tiers** — binary pass/fail rules are errors; spectrum rules (contrast, target size) fall back to warnings below a configurable threshold instead of a hard failure.

See [wcag2.2-rules.md](./wcag2.2-rules.md) for the full list of rules and the WCAG 2.2 success criteria they check.

## Installation

Not published anywhere yet — build from source:

```bash
cargo build --release
```

The binary is written to `target/release/wcag-checker`.

## Usage

```cmd
Check a web page against a subset of WCAG 2.2 rules

Usage: wcag-checker [OPTIONS] <PATH_OR_URL>

Arguments:
  <PATH_OR_URL>  HTML source: an http(s) URL, or a path to a local HTML file

Options:
  -v, --verbose
          Enable stdout output
      --contrast-threshold <CONTRAST_THRESHOLD>
          Minimum contrast ratio for normal-size text (WCAG 1.4.3 default: 4.5) [default: 4.5]
      --large-text-contrast-threshold <LARGE_TEXT_CONTRAST_THRESHOLD>
          Minimum contrast ratio for large-scale text (WCAG 1.4.3 default: 3.0) [default: 3]
      --target-size-threshold <TARGET_SIZE_THRESHOLD>
          Minimum pointer target size in CSS pixels (WCAG 2.5.8 default: 24.0) [default: 24]
      --full-site-scan
          When given a URL, also scan every same-domain page reachable through its links
      --full-site-scan-max-pages <FULL_SITE_SCAN_MAX_PAGES>
          Maximum number of HTML pages checked during a full-site scan (default: no limit)
  -h, --help
          Print help
  -V, --version
          Print version
```

**Examples:**

```bash
# From a local file
wcag-checker page.html

# From a URL
wcag-checker https://example.com

# Loosen the contrast threshold to AA-large only
wcag-checker page.html --contrast-threshold 3.0

# Tighten the target size to the enhanced (AAA) 44px guidance
wcag-checker page.html --target-size-threshold 44

# Crawl and check every page of a site, capped at 200 checked HTML pages
wcag-checker https://example.com --full-site-scan --full-site-scan-max-pages 200
```

**Full-site scan** (`--full-site-scan`, URLs only -- ignored for local files):

- Follows every `<a href>` / `<area href>` on the same host (relative or absolute), in parallel. Each path is downloaded at most once, redirect targets included; query strings and fragments are ignored when deciding whether a path was already fetched.
- `--full-site-scan-max-pages` counts only HTML pages that are actually rendered and checked; JSON/XML/PDF responses, HTTP errors, and already-fetched paths don't use up the budget.
- Redirects are followed only while they stay on the same host; a redirect to another host is skipped.
- Responses that aren't HTML (JSON, XML, PDF, images, ...) are skipped silently.
- `4XX`/`5XX` responses are reported as `HTTP` errors, and network failures as `FETCH` errors.
- Rate limits are respected: `429` (and `503` with `Retry-After`) are retried up to 3 times after the `Retry-After` delay (1s when it is `0` or missing), and `RateLimit-*`/`X-RateLimit-*` headers pause all requests once the quota runs out.
- Each report line includes the URL path of the page it belongs to, e.g. `[ERROR] H57 /about: ...`.

**Exit codes:**

| Code | Meaning                                                    |
| ---- | ----------------------------------------------------------- |
| `0`  | No findings                                                  |
| `1`  | At least one error (a binary pass/fail rule failed)          |
| `2`  | Only warnings (a spectrum rule fell below its threshold)     |

## Architecture

```
src/
├── main.rs                 # fetch/read -> render -> run rules -> report -> exit code
├── cli.rs                  # CLI arguments (clap)
├── error.rs                 # CheckerError
├── logging.rs                # tracing subscriber setup
├── report.rs                 # stdout rendering + exit code derivation
│
├── site/                    # --full-site-scan
│   ├── crawler.rs             # parallel crawl: visited set, page budget, per-page checks
│   ├── links.rs               # href extraction, same-domain check, path-only visit key
│   └── request_helper.rs      # request(): rate limits, retries, same-host redirects, content type
│
├── page/                    # everything that turns a URL/file into a RenderedPage
│   ├── fetch.rs               # URL fetch (reqwest) vs local file read
│   ├── render.rs              # embedded litehtml engine -> RenderedPage
│   ├── model.rs               # RenderedPage / RenderedElement / ElementRef + query helpers
│   └── testutil.rs            # page_from_html() test helper (scraper, dev-dependency only)
│
└── rules/                   # one module per WCAG rule area, one Rule fn per rule
    ├── images.rs, forms.rs, headings.rs, language.rs, links.rs, contrast.rs,
    │   tables.rs, aria.rs, multimedia.rs, focus.rs, navigation.rs, target_size.rs
    └── mod.rs                  # rule registry + parallel dispatch via tokio::spawn

vendor/
├── litehtml-sys/            # vendored, patched raw FFI bindings (see vendor/PATCHES.md)
└── litehtml/                # vendored, patched safe Rust wrapper
```

Every rule has the same shape — `fn(&RenderedPage, &CheckOptions) -> Vec<Finding>` — and reads from the same rendered snapshot; there is no distinction between rules that need computed style/layout and rules that only need DOM/attribute data.

## Tests

- Unit tests build `RenderedPage` fixtures from plain HTML via `page::testutil::page_from_html`, without invoking the real rendering engine.
- Integration tests (`tests/integration.rs`) run the compiled binary against real fixtures under `tests/assets/` through the actual embedded engine.

Run: `cargo test`
