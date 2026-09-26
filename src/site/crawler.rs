//! Full-site scan: starting from one URL, scan every same-host page
//! reachable through `href`s, in parallel, each page exactly once.
//!
//! Pages are crawled breadth-first, one level at a time. A level's links are
//! ordered by where they were found (the discovering page's position in its
//! own level, then document order), not by which download happened to
//! finish first -- so with `--full-site-scan-max-pages`, the same site
//! always yields the same set of checked pages. Within a level, pages are
//! fetched in parallel, but never more at once than the page budget has
//! slots left, so every HTML page downloaded is guaranteed to be checked.
//!
//! Whether a path was already fetched is ultimately decided by `request()`
//! itself, right before each download (see [`RequestContext`]); `queued`
//! here only stops the same link from being scheduled twice.

use std::collections::{HashSet, VecDeque};
use std::num::NonZeroUsize;
use std::sync::Arc;

use tokio::task::JoinSet;
use tracing::{error, info};
use url::Url;

use crate::error::CheckerError;
use crate::page::{self, RenderedPage};
use crate::rules::{self, CheckOptions, Finding};

use super::links::{extract_links, is_same_domain, visit_key};
use super::request_helper::{RedirectScope, RequestContext, RequestOutcome, request};

const MAX_CONCURRENT_PAGES: usize = 8;

#[derive(Default)]
struct PageScan {
    findings: Vec<Finding>,
    links: Vec<Url>,
    /// Set only when the page was rendered and checked -- the only thing
    /// `--full-site-scan-max-pages` counts.
    checked_url: Option<Url>,
}

struct Crawl {
    /// Its host is the one links are followed on. Starts as the CLI's URL,
    /// then becomes wherever that URL finally redirected to.
    origin: Url,
    origin_resolved: bool,
    context: Arc<RequestContext>,
    options: Arc<CheckOptions>,
    max_pages: Option<NonZeroUsize>,
    queued: HashSet<String>,
    level: VecDeque<Url>,
    /// Links found by pages of the current level, tagged with the position
    /// of the page that found them.
    discovered: Vec<(usize, Vec<Url>)>,
    started_in_level: usize,
    tasks: JoinSet<(usize, PageScan)>,
    checked: usize,
    findings: Vec<Finding>,
}

impl Crawl {
    fn new(
        start: Url,
        options: Arc<CheckOptions>,
        max_pages: Option<NonZeroUsize>,
    ) -> Result<Self, CheckerError> {
        Ok(Crawl {
            origin: start.clone(),
            origin_resolved: false,
            context: Arc::new(RequestContext::new()?),
            options,
            max_pages,
            queued: HashSet::from([visit_key(&start)]),
            level: VecDeque::from([start]),
            discovered: Vec::new(),
            started_in_level: 0,
            tasks: JoinSet::new(),
            checked: 0,
            findings: Vec::new(),
        })
    }

    async fn run(mut self) -> Vec<Finding> {
        loop {
            self.start_tasks();
            match self.tasks.join_next().await {
                Some(Ok((position, scan))) => self.collect(position, scan),
                Some(Err(join_error)) => error!("page scan task panicked: {join_error}"),
                None if self.advance_level() => {}
                None => break,
            }
        }
        info!("Checked {} HTML page(s)", self.checked);
        if self.checked == 0 && self.findings.is_empty() {
            self.findings.push(nothing_checked(&self.origin));
        }
        // A stable sort keeps each page's findings in rule order.
        self.findings.sort_by(|a, b| a.page.cmp(&b.page));
        self.findings
    }

    fn start_tasks(&mut self) {
        while self.capacity() > 0
            && let Some(url) = self.level.pop_front()
        {
            let position = self.started_in_level;
            self.started_in_level += 1;
            let scope = self.redirect_scope();
            let context = Arc::clone(&self.context);
            let options = Arc::clone(&self.options);
            self.tasks
                .spawn(async move { (position, scan_page(&context, options, url, &scope).await) });
        }
    }

    /// Every page in flight might turn out to be HTML, so it already holds
    /// one of the budget's remaining slots.
    fn capacity(&self) -> usize {
        let in_flight = self.tasks.len();
        let budget_left = self.max_pages.map_or(usize::MAX, |max| {
            max.get().saturating_sub(self.checked + in_flight)
        });
        MAX_CONCURRENT_PAGES
            .saturating_sub(in_flight)
            .min(budget_left)
    }

    fn budget_exhausted(&self) -> bool {
        self.max_pages.is_some_and(|max| self.checked >= max.get())
    }

    fn redirect_scope(&self) -> RedirectScope {
        match self.origin_resolved {
            true => RedirectScope::SameHost(self.origin.host_str().unwrap_or_default().to_string()),
            false => RedirectScope::AnyHost,
        }
    }

    fn collect(&mut self, position: usize, scan: PageScan) {
        self.findings.extend(scan.findings);
        if let Some(checked_url) = scan.checked_url {
            self.checked += 1;
            if !self.origin_resolved {
                self.origin = checked_url;
            }
        }
        self.origin_resolved = true;
        self.discovered.push((position, scan.links));
    }

    /// Builds the next level from everything the finished one discovered;
    /// `false` when there is nothing left to crawl.
    fn advance_level(&mut self) -> bool {
        if self.budget_exhausted() {
            return false;
        }
        let mut discovered = std::mem::take(&mut self.discovered);
        discovered.sort_by_key(|(position, _)| *position);
        self.started_in_level = 0;
        for link in discovered.into_iter().flat_map(|(_, links)| links) {
            if self.should_visit(&link) {
                self.level.push_back(link);
            }
        }
        !self.level.is_empty()
    }

    fn should_visit(&mut self, url: &Url) -> bool {
        is_same_domain(url, &self.origin)
            && !self.context.has_fetched(url)
            && self.queued.insert(visit_key(url))
    }
}

pub async fn crawl(
    start: Url,
    options: Arc<CheckOptions>,
    max_pages: Option<NonZeroUsize>,
) -> Result<Vec<Finding>, CheckerError> {
    Ok(Crawl::new(start, options, max_pages)?.run().await)
}

/// Otherwise a start URL that isn't HTML would read as a clean
/// `0 error(s)` pass. (HTTP/network failures already report their own
/// error.)
fn nothing_checked(origin: &Url) -> Finding {
    Finding::error(
        "SCAN",
        format!("no HTML page could be checked starting from {origin}"),
    )
    .on_page(origin)
}

async fn scan_page(
    context: &RequestContext,
    options: Arc<CheckOptions>,
    url: Url,
    scope: &RedirectScope,
) -> PageScan {
    match request(context, &url, scope).await {
        RequestOutcome::Html { final_url, body } => check_page(options, final_url, body).await,
        RequestOutcome::AlreadyFetched(hop) => {
            info!("Skipping {url}: redirects to {hop}, which was already fetched");
            PageScan::default()
        }
        RequestOutcome::NotRenderable => {
            info!("Skipping {url}: not an HTML page");
            PageScan::default()
        }
        RequestOutcome::OffDomainRedirect(target) => {
            info!("Skipping {url}: redirects off-domain to {target}");
            PageScan::default()
        }
        RequestOutcome::HttpError(status) => {
            failure(Finding::error("HTTP", format!("HTTP {status}")).on_page(&url))
        }
        RequestOutcome::Transport(err) => {
            failure(Finding::error("FETCH", err.to_string()).on_page(&url))
        }
    }
}

async fn check_page(options: Arc<CheckOptions>, url: Url, body: String) -> PageScan {
    // litehtml rendering is synchronous and CPU-bound; keep it off the async
    // worker threads that are driving the other pages' requests.
    let rendered = tokio::task::spawn_blocking(move || page::render(&body)).await;
    let page = match rendered {
        Ok(Ok(page)) => Arc::new(RenderedPage {
            url: Some(url.clone()),
            ..page
        }),
        Ok(Err(err)) => return failure(Finding::error("RENDER", err.to_string()).on_page(&url)),
        Err(join_error) => {
            return failure(Finding::error("RENDER", join_error.to_string()).on_page(&url));
        }
    };

    let links = extract_links(&page, &url);
    let findings = rules::run_all(page, options)
        .await
        .into_iter()
        .map(|finding| finding.on_page(&url))
        .collect();
    PageScan {
        findings,
        links,
        checked_url: Some(url),
    }
}

fn failure(finding: Finding) -> PageScan {
    PageScan {
        findings: vec![finding],
        ..PageScan::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use crate::rules::Severity;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const CLEAN_BODY: &str = r##"<html lang="en"><body>
        <a href="#main">Skip to main content</a><h1 id="main">Title</h1>"##;

    fn options() -> Arc<CheckOptions> {
        Arc::new(CheckOptions {
            contrast_threshold: 4.5,
            large_text_contrast_threshold: 3.0,
            // Unstyled links render below 24px; irrelevant to crawl behavior.
            target_size_threshold: 0.0,
        })
    }

    fn page_linking_to(hrefs: &[&str]) -> ResponseTemplate {
        let links: String = hrefs
            .iter()
            .map(|href| format!(r#"<a href="{href}">Go to {href}</a>"#))
            .collect();
        let body = format!("{CLEAN_BODY}{links}</body></html>");
        ResponseTemplate::new(200).set_body_raw(body, "text/html")
    }

    async fn serve(server: &MockServer, route: &str, response: ResponseTemplate, times: u64) {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(response)
            .expect(times)
            .mount(server)
            .await;
    }

    /// Uses `localhost` so a second mock server reached via `127.0.0.1` is a
    /// genuinely different host.
    fn start_url(server: &MockServer, route: &str) -> Url {
        Url::parse(&format!(
            "http://localhost:{}{route}",
            server.address().port()
        ))
        .unwrap()
    }

    async fn run(server: &MockServer, max_pages: Option<usize>) -> Vec<Finding> {
        let max_pages = max_pages.and_then(NonZeroUsize::new);
        crawl(start_url(server, "/"), options(), max_pages)
            .await
            .unwrap()
    }

    fn pages_with_findings(findings: &[Finding]) -> Vec<&str> {
        let mut pages: Vec<&str> = findings.iter().filter_map(Finding::page_path).collect();
        pages.dedup();
        pages
    }

    #[tokio::test]
    async fn scans_each_page_once_despite_cycles() {
        let server = MockServer::start().await;
        serve(&server, "/", page_linking_to(&["/", "/a", "/b"]), 1).await;
        serve(&server, "/a", page_linking_to(&["/b", "/"]), 1).await;
        serve(&server, "/b", page_linking_to(&["/a", "/b"]), 1).await;

        let findings = run(&server, None).await;
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[tokio::test]
    async fn query_strings_do_not_create_new_pages() {
        let server = MockServer::start().await;
        serve(
            &server,
            "/",
            page_linking_to(&["/list?page=1", "/list?page=2#x"]),
            1,
        )
        .await;
        serve(&server, "/list", page_linking_to(&["/list?page=3"]), 1).await;

        run(&server, None).await;
    }

    #[tokio::test]
    async fn never_leaves_the_domain() {
        let server = MockServer::start().await;
        let external = MockServer::start().await;
        let external_page = format!("{}/elsewhere", external.uri());
        serve(&server, "/", page_linking_to(&[&external_page]), 1).await;
        serve(&external, "/elsewhere", page_linking_to(&[]), 0).await;

        run(&server, None).await;
    }

    #[tokio::test]
    async fn follows_same_domain_redirects_and_tags_the_final_path() {
        let server = MockServer::start().await;
        serve(&server, "/", page_linking_to(&["/old"]), 1).await;
        let redirect = ResponseTemplate::new(301).insert_header("location", "/new");
        serve(&server, "/old", redirect, 1).await;
        let missing_h1 = ResponseTemplate::new(200).set_body_raw("<p>no heading</p>", "text/html");
        serve(&server, "/new", missing_h1, 1).await;

        let findings = run(&server, None).await;
        assert_eq!(pages_with_findings(&findings), ["/new"]);
    }

    #[tokio::test]
    async fn redirect_to_an_already_fetched_page_is_not_fetched_again() {
        let server = MockServer::start().await;
        serve(&server, "/", page_linking_to(&["/alias"]), 1).await;
        let redirect = ResponseTemplate::new(302).insert_header("location", "/");
        serve(&server, "/alias", redirect, 1).await;

        let findings = run(&server, None).await;
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[tokio::test]
    async fn off_domain_redirects_are_skipped_silently() {
        let server = MockServer::start().await;
        let external = MockServer::start().await;
        serve(&server, "/", page_linking_to(&["/away"]), 1).await;
        let redirect = ResponseTemplate::new(302).insert_header("location", external.uri());
        serve(&server, "/away", redirect, 1).await;
        serve(&external, "/", page_linking_to(&[]), 0).await;

        let findings = run(&server, None).await;
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[tokio::test]
    async fn http_errors_are_reported_with_their_path() {
        let server = MockServer::start().await;
        serve(&server, "/", page_linking_to(&["/missing", "/boom"]), 1).await;
        serve(&server, "/missing", ResponseTemplate::new(404), 1).await;
        serve(&server, "/boom", ResponseTemplate::new(500), 1).await;

        let findings = run(&server, None).await;
        assert_eq!(findings.len(), 2, "{findings:?}");
        assert!(findings.iter().all(|f| f.rule_id == "HTTP"));
        assert!(findings.iter().all(|f| f.severity == Severity::Error));
        assert_eq!(pages_with_findings(&findings), ["/boom", "/missing"]);
        assert!(findings[1].message.contains("404"), "{findings:?}");
    }

    #[tokio::test]
    async fn api_responses_are_ignored() {
        let server = MockServer::start().await;
        serve(
            &server,
            "/",
            page_linking_to(&["/api/items", "/feed.xml"]),
            1,
        )
        .await;
        let json = ResponseTemplate::new(200).set_body_raw(r#"{"a":1}"#, "application/json");
        serve(&server, "/api/items", json, 1).await;
        let xml = ResponseTemplate::new(200).set_body_raw("<rss/>", "application/rss+xml");
        serve(&server, "/feed.xml", xml, 1).await;

        let findings = run(&server, None).await;
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[tokio::test]
    async fn rate_limited_pages_are_retried() {
        let server = MockServer::start().await;
        serve(&server, "/", page_linking_to(&["/slow"]), 1).await;
        Mock::given(method("GET"))
            .and(path("/slow"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
            .up_to_n_times(1)
            .with_priority(1)
            .expect(1)
            .mount(&server)
            .await;
        serve(&server, "/slow", page_linking_to(&[]), 1).await;

        let findings = run(&server, None).await;
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[tokio::test]
    async fn max_pages_counts_only_processed_html_pages() {
        let server = MockServer::start().await;
        serve(
            &server,
            "/",
            page_linking_to(&["/api", "/doc.pdf", "/gone", "/a"]),
            1,
        )
        .await;
        let json = ResponseTemplate::new(200).set_body_raw("{}", "application/json");
        serve(&server, "/api", json, 1).await;
        let pdf = ResponseTemplate::new(200).set_body_raw("%PDF", "application/pdf");
        serve(&server, "/doc.pdf", pdf, 1).await;
        serve(&server, "/gone", ResponseTemplate::new(410), 1).await;
        serve(&server, "/a", page_linking_to(&["/b"]), 1).await;
        serve(&server, "/b", page_linking_to(&[]), 0).await;

        run(&server, Some(2)).await;
    }

    #[tokio::test]
    async fn max_pages_is_exact_under_parallel_fetches() {
        let server = MockServer::start().await;
        let siblings = ["/a", "/b", "/c", "/d", "/e"];
        serve(&server, "/", page_linking_to(&siblings), 1).await;
        let unlabelled = ResponseTemplate::new(200).set_body_raw("<p>no lang</p>", "text/html");
        for sibling in siblings {
            Mock::given(method("GET"))
                .and(path(sibling))
                .respond_with(unlabelled.clone())
                .mount(&server)
                .await;
        }

        let findings = run(&server, Some(3)).await;
        assert_eq!(pages_with_findings(&findings).len(), 2, "{findings:?}");
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 3, "downloaded pages beyond the budget");
    }

    #[tokio::test]
    async fn non_html_downloads_hand_their_budget_slot_back() {
        let server = MockServer::start().await;
        let links = ["/api/1", "/api/2", "/api/3", "/gone", "/a", "/b"];
        serve(&server, "/", page_linking_to(&links), 1).await;
        let json = ResponseTemplate::new(200).set_body_raw("{}", "application/json");
        for api in ["/api/1", "/api/2", "/api/3"] {
            serve(&server, api, json.clone(), 1).await;
        }
        serve(&server, "/gone", ResponseTemplate::new(404), 1).await;
        let unlabelled = ResponseTemplate::new(200).set_body_raw("<p>no lang</p>", "text/html");
        for sibling in ["/a", "/b"] {
            Mock::given(method("GET"))
                .and(path(sibling))
                .respond_with(unlabelled.clone())
                .mount(&server)
                .await;
        }

        // One slot left after `/`: the API and 404 downloads must each hand
        // it back, and exactly one of `/a` / `/b` must get to use it.
        let findings = run(&server, Some(2)).await;
        let checked: Vec<&str> = pages_with_findings(&findings)
            .into_iter()
            .filter(|page| ["/a", "/b"].contains(page))
            .collect();
        assert_eq!(checked.len(), 1, "{findings:?}");
        let requests = server.received_requests().await.unwrap();
        let html_requests = requests
            .iter()
            .filter(|r| ["/a", "/b"].contains(&r.url.path()))
            .count();
        assert_eq!(html_requests, 1, "downloaded HTML beyond the budget");
    }

    #[tokio::test]
    async fn start_url_redirecting_to_another_host_crawls_that_host() {
        let server = MockServer::start().await;
        let moved = MockServer::start().await;
        let redirect =
            ResponseTemplate::new(301).insert_header("location", format!("{}/", moved.uri()));
        serve(&server, "/", redirect, 1).await;
        let back_to_old_host = start_url(&server, "/old-host-page").to_string();
        serve(&moved, "/", page_linking_to(&["/b", &back_to_old_host]), 1).await;
        serve(&moved, "/b", page_linking_to(&[]), 1).await;
        serve(&server, "/old-host-page", page_linking_to(&[]), 0).await;

        let findings = run(&server, None).await;
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[tokio::test]
    async fn start_url_that_is_not_html_is_an_error_not_a_clean_pass() {
        let server = MockServer::start().await;
        let json = ResponseTemplate::new(200).set_body_raw("{}", "application/json");
        serve(&server, "/", json, 1).await;

        let findings = run(&server, None).await;
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule_id, "SCAN");
        assert_eq!(findings[0].severity, Severity::Error);
    }

    #[tokio::test]
    async fn duplicate_links_are_scheduled_once() {
        let start = Url::parse("http://localhost/").unwrap();
        let mut crawl = Crawl::new(start.clone(), options(), None).unwrap();
        crawl.origin_resolved = true;
        crawl.level.clear();
        let link = |href: &str| start.join(href).unwrap();
        crawl.discovered = vec![
            (1, vec![link("/a"), link("/b#top")]),
            (0, vec![link("/b"), link("/a?x=1"), link("/a"), link("/")]),
        ];

        assert!(crawl.advance_level());
        let level: Vec<&str> = crawl.level.iter().map(Url::path).collect();
        assert_eq!(level, ["/b", "/a"]);
    }

    #[tokio::test]
    async fn page_selection_under_a_budget_ignores_download_speed() {
        let server = MockServer::start().await;
        serve(&server, "/", page_linking_to(&["/slow", "/fast"]), 1).await;
        let slow = page_linking_to(&["/from-slow"]).set_delay(Duration::from_millis(300));
        serve(&server, "/slow", slow, 1).await;
        serve(&server, "/fast", page_linking_to(&["/from-fast"]), 1).await;
        serve(&server, "/from-slow", page_linking_to(&[]), 1).await;
        serve(&server, "/from-fast", page_linking_to(&[]), 0).await;

        // `/fast` finishes first, but `/slow` comes first in document order,
        // so its link is the one that gets the last slot.
        run(&server, Some(4)).await;
    }

    #[tokio::test]
    async fn start_page_errors_are_reported_too() {
        let server = MockServer::start().await;
        serve(&server, "/", ResponseTemplate::new(503), 1).await;

        let findings = run(&server, None).await;
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].page_path(), Some("/"));
    }
}
