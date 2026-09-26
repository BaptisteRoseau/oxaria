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
          Also log info-level messages
  -q, --quiet
          Print nothing on stdout (findings, summary, and logs); only the exit code and the --report-* files remain
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
      --report-gitlab <FILE>
          Write a GitLab Code Quality report (artifacts:reports:codequality) to FILE ('-' for stdout)
      --report-github <FILE>
          Append a GitHub Actions job summary (Markdown) to FILE ('-' for stdout); defaults to $GITHUB_STEP_SUMMARY, so it is written automatically inside GitHub Actions [env: GITHUB_STEP_SUMMARY=]
      --report-jenkins <FILE>
          Write a Jenkins Warnings NG report (recordIssues tool: issues()) to FILE ('-' for stdout)
      --report-junit <FILE>
          Write a JUnit XML report (Jenkins junit step, GitLab artifacts:reports:junit) to FILE ('-' for stdout)
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

# CI: GitLab Code Quality + JUnit reports, nothing on stdout
wcag-checker page.html -q --report-gitlab gl-code-quality.json --report-junit junit.xml
```

**Full-site scan** (`--full-site-scan`, URLs only -- ignored for local files):

- Follows every `<a href>` / `<area href>` on the same host (relative or absolute), in parallel. Each path is downloaded at most once, redirect targets included; query strings and fragments are ignored when deciding whether a path was already fetched.
- `--full-site-scan-max-pages` counts only HTML pages that are actually rendered and checked; JSON/XML/PDF responses, HTTP errors, and already-fetched paths don't use up the budget.
- Redirects are followed only while they stay on the same host; a redirect to another host is skipped. The one exception is the start URL: it may redirect anywhere (e.g. `example.com` → `www.example.com`), and the crawl then continues on the host it landed on.
- Pages are crawled breadth-first, level by level, so with `--full-site-scan-max-pages` the same site always yields the same set of checked pages.
- Responses that aren't HTML (JSON, XML, PDF, images, ...) are skipped silently.
- `4XX`/`5XX` responses are reported as `HTTP` errors, and network failures as `FETCH` errors. A scan that couldn't check any HTML page (e.g. the start URL returns JSON) reports a `SCAN` error rather than a clean pass.
- Requests identify themselves as `wcag-checker/<version>` and ask for HTML (`Accept: text/html,…`), in both single-page and full-site mode.
- Rate limits are respected: `429` (and `503` with `Retry-After`) are retried up to 3 times after the `Retry-After` delay (1s when it is `0` or missing), and `RateLimit-*`/`X-RateLimit-*` headers pause all requests once the quota runs out.
- Each report line includes the URL path of the page it belongs to, e.g. `[ERROR] H57 /about: ...`. In the `--report-*` files, a crawled page is located by the full URL it was actually served from (after redirects).

**Element paths:** findings about one element end with a CSS-selector-like path to it, e.g. `[WARN]  TGT001: <button> target size is 19px, below the required 24px (at body > header > nav > ul > li:nth-of-type(2) > button)`. The path stops at the nearest element with a unique id (`input#signup-email`). Page-level findings (`H42`, `H57`, `G1` without a skip link, ...), stylesheet findings (`G195`), and findings spanning several elements (`IDS001`, `LNK001`) have none.

**CI reports** (`--report-*`):

Each `--report-<kind> <FILE>` writes the findings in a format a CI platform reads natively (`-` writes to stdout; an empty value writes nothing). Several can be combined; they are written in parallel once the check is done. The usual stdout output is still printed unless `-q/--quiet` is given.

| Flag                | Format                                                                                                    | Read by                                                        |
| ------------------- | --------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| `--report-gitlab`   | [Code Quality JSON](https://docs.gitlab.com/ci/testing/code_quality/)                                      | GitLab `artifacts:reports:codequality` (MR widget, diff annotations) |
| `--report-github`   | [Job summary](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#adding-a-job-summary) (GitHub-flavored Markdown table) | GitHub Actions run summary page. Defaults to `$GITHUB_STEP_SUMMARY`, so it's written automatically in GitHub Actions |
| `--report-jenkins`  | [Warnings NG](https://github.com/jenkinsci/warnings-ng-plugin/blob/main/doc/Documentation.md) native JSON | Jenkins `recordIssues(tool: issues(pattern: '…'))`              |
| `--report-junit`    | JUnit XML (errors fail, warnings pass with a `system-out`)                                                 | Jenkins `junit`, GitLab `artifacts:reports:junit`               |

- Errors map to GitLab `major` / Jenkins `ERROR` / GitHub `❌ Error`; warnings to `minor` / `NORMAL` / `⚠️ Warning`.
- The GitHub summary is **appended** to its file, as GitHub expects for `$GITHUB_STEP_SUMMARY` (other commands of the same step may write to it too). It lists every finding, errors first; only past GitHub's 1 MiB summary limit are the remaining ones replaced by a count. Annotations aren't used: GitHub shows only 10 errors + 10 warnings per step. To turn the summary off inside GitHub Actions, set `GITHUB_STEP_SUMMARY=` for the command.
- Warnings NG users should read `--report-jenkins`, not the JUnit report: Warnings NG's JUnit parser drops warnings and can't locate issues on URLs.
- The rendering engine exposes no source positions, so every issue points at line 1 of the checked file (or, for URLs, of the page URL).
- Two reports writing to the same file (including the same path spelled differently), two reports on stdout, or a report overwriting the checked file are rejected with exit code `1` before anything is checked.
- A fatal error (unreadable file, unreachable URL) is reported as an `INPUT`/`FETCH`/`RENDER` issue, so CI still gets a report explaining the failure.

```yaml
# .gitlab-ci.yml
wcag:
  script: wcag-checker public/index.html --report-gitlab gl-code-quality.json --report-junit junit.xml
  artifacts:
    when: always
    reports:
      codequality: gl-code-quality.json
      junit: junit.xml
```

```yaml
# GitHub Actions step: the job summary goes to $GITHUB_STEP_SUMMARY with no flag
- run: wcag-checker public/index.html -q
```

```groovy
// Jenkinsfile
sh 'wcag-checker public/index.html --report-jenkins wcag.json || true'
recordIssues(tool: issues(pattern: 'wcag.json', id: 'wcag', name: 'WCAG'))
```

**Exit codes:**

| Code | Meaning                                                    |
| ---- | ----------------------------------------------------------- |
| `0`  | No findings                                                  |
| `1`  | At least one error (a binary pass/fail rule failed)          |
| `2`  | Only warnings (a spectrum rule fell below its threshold)     |

## Architecture

```
src/
├── main.rs                 # validate reports -> fetch/read -> render -> run rules -> report -> exit code
├── cli.rs                  # CLI arguments (clap)
├── error.rs                 # CheckerError
├── logging.rs                # tracing subscriber setup
│
├── reporters/               # --report-* and the stdout output
│   ├── model.rs               # Report / Issue / Location: findings + location + fingerprint, exit code
│   ├── reporter.rs            # Reporter trait: render a Report to one format
│   ├── text.rs                # default human-readable stdout output
│   ├── gitlab.rs, github.rs, jenkins.rs, junit.rs   # one Reporter per CI format
│   └── output.rs              # destinations, up-front conflict check, parallel writes
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
    │   tables.rs, aria.rs, multimedia.rs, focus.rs, navigation.rs, target_size.rs,
    │   document.rs, frames.rs, timing.rs, autocomplete.rs, label_in_name.rs,
    │   scripting.rs, authentication.rs
    └── mod.rs                  # rule registry + parallel dispatch via spawn_blocking

vendor/
├── litehtml-sys/            # vendored, patched raw FFI bindings (see vendor/PATCHES.md)
└── litehtml/                # vendored, patched safe Rust wrapper
```

Every rule has the same shape — `fn(&RenderedPage, &CheckOptions) -> Vec<Finding>` — and reads from the same rendered snapshot; there is no distinction between rules that need computed style/layout and rules that only need DOM/attribute data.

## Tests

- Unit tests build `RenderedPage` fixtures from plain HTML via `page::testutil::page_from_html`, without invoking the real rendering engine.
- Integration tests (`tests/integration.rs`) run the compiled binary against real fixtures under `tests/assets/` through the actual embedded engine.

Run: `cargo test`
