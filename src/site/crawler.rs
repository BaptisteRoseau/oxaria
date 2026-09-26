//! Full-site scan: starting from one URL, scan every same-host page
//! reachable through `href`s, in parallel, each page exactly once.
//!
//! Whether a path was already fetched is decided by `request()` itself,
//! right before each download (see [`RequestContext`]); the orchestrator
//! only skips links it can already see are fetched, to avoid spawning
//! tasks that would do nothing.

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::task::JoinSet;
use tracing::{error, info};
use url::Url;

use crate::error::CheckerError;
use crate::page;
use crate::rules::{self, CheckOptions, Finding};

use super::links::{extract_links, is_same_domain, visit_key};
use super::request_helper::{RequestContext, RequestOutcome, request};

const MAX_CONCURRENT_PAGES: usize = 8;

#[derive(Default)]
struct PageScan {
    findings: Vec<Finding>,
    links: Vec<Url>,
}

struct Shared {
    context: RequestContext,
    options: Arc<CheckOptions>,
    permits: Semaphore,
    budget: PageBudget,
}

/// `--full-site-scan-max-pages`: counts only HTML pages that are actually
/// rendered and checked -- not API/PDF/other responses, errors, or paths
/// skipped because they were already fetched.
///
/// A slot is reserved *before* downloading, so no more pages are downloaded
/// at once than there are slots left. A download that turns out not to be
/// checkable HTML hands its slot back for a waiting task. Without this, every
/// download already in flight when the budget filled up would be wasted.
struct PageBudget {
    max_pages: Option<NonZeroUsize>,
    /// `None` when there is no limit.
    slots: Option<Semaphore>,
    processed: AtomicUsize,
}

/// Holds a budget slot for the duration of one download; dropping it without
/// [`PageBudget::consume`] returns the slot.
struct Reservation<'a>(Option<SemaphorePermit<'a>>);

impl PageBudget {
    fn new(max_pages: Option<NonZeroUsize>) -> Self {
        PageBudget {
            max_pages,
            slots: max_pages.map(|max| Semaphore::new(max.get())),
            processed: AtomicUsize::new(0),
        }
    }

    fn exhausted(&self) -> bool {
        self.slots.as_ref().is_some_and(Semaphore::is_closed)
    }

    /// Waits for a free slot; `None` once the budget is used up.
    async fn reserve(&self) -> Option<Reservation<'_>> {
        match &self.slots {
            Some(slots) => slots.acquire().await.ok().map(|p| Reservation(Some(p))),
            None => Some(Reservation(None)),
        }
    }

    /// Keeps the slot for good. Closing the semaphore once the last slot is
    /// consumed wakes every task still waiting in `reserve`, which would
    /// otherwise wait forever on a semaphore with no permits left.
    fn consume(&self, reservation: Reservation) {
        if let Some(permit) = reservation.0 {
            permit.forget();
        }
        let processed = self.processed.fetch_add(1, Ordering::SeqCst) + 1;
        if self.max_pages.is_some_and(|max| processed >= max.get())
            && let Some(slots) = &self.slots
        {
            slots.close();
        }
    }

    fn processed(&self) -> usize {
        self.processed.load(Ordering::SeqCst)
    }
}

struct Crawl {
    origin: Url,
    shared: Arc<Shared>,
    tasks: JoinSet<PageScan>,
}

impl Crawl {
    fn enqueue(&mut self, url: Url) {
        let skip = !is_same_domain(&url, &self.origin)
            || self.shared.budget.exhausted()
            || self.shared.context.has_fetched(&url);
        if !skip {
            self.tasks.spawn(scan_page(Arc::clone(&self.shared), url));
        }
    }
}

pub async fn crawl(
    start: Url,
    options: Arc<CheckOptions>,
    max_pages: Option<NonZeroUsize>,
) -> Result<Vec<Finding>, CheckerError> {
    let shared = Shared {
        context: RequestContext::new(start.host_str().unwrap_or_default())?,
        options,
        permits: Semaphore::new(MAX_CONCURRENT_PAGES),
        budget: PageBudget::new(max_pages),
    };
    let mut crawl = Crawl {
        origin: start.clone(),
        shared: Arc::new(shared),
        tasks: JoinSet::new(),
    };

    crawl.enqueue(start);
    let mut findings = Vec::new();
    while let Some(joined) = crawl.tasks.join_next().await {
        match joined {
            Ok(scan) => {
                findings.extend(scan.findings);
                scan.links.into_iter().for_each(|link| crawl.enqueue(link));
            }
            Err(join_error) => error!("page scan task panicked: {join_error}"),
        }
    }
    info!("Checked {} HTML page(s)", crawl.shared.budget.processed());

    // Tasks finish in arbitrary order; a stable sort keeps each page's
    // findings in rule order while making the overall report deterministic.
    findings.sort_by(|a, b| a.page.cmp(&b.page));
    Ok(findings)
}

async fn scan_page(shared: Arc<Shared>, url: Url) -> PageScan {
    // Budget before concurrency: a task waiting for a budget slot then
    // doesn't hold one of the concurrency permits other tasks could use.
    let Some(reservation) = shared.budget.reserve().await else {
        return PageScan::default();
    };
    let _permit = shared
        .permits
        .acquire()
        .await
        .expect("concurrency semaphore is never closed");
    let path = visit_key(&url);

    match request(&shared.context, &url).await {
        RequestOutcome::Html { final_url, body } => {
            shared.budget.consume(reservation);
            check_page(&shared, final_url, body).await
        }
        RequestOutcome::AlreadyFetched => PageScan::default(),
        RequestOutcome::NotRenderable => {
            info!("Skipping {url}: not an HTML page");
            PageScan::default()
        }
        RequestOutcome::OffDomainRedirect(target) => {
            info!("Skipping {url}: redirects off-domain to {target}");
            PageScan::default()
        }
        RequestOutcome::HttpError(status) => {
            failure(Finding::error("HTTP", format!("HTTP {status}")).on_page(path))
        }
        RequestOutcome::Transport(err) => {
            failure(Finding::error("FETCH", err.to_string()).on_page(path))
        }
    }
}

async fn check_page(shared: &Shared, url: Url, body: String) -> PageScan {
    let path = visit_key(&url);
    // litehtml rendering is synchronous and CPU-bound; keep it off the async
    // worker threads that are driving the other pages' requests.
    let rendered = tokio::task::spawn_blocking(move || page::render(&body)).await;
    let page = match rendered {
        Ok(Ok(page)) => Arc::new(page),
        Ok(Err(err)) => return failure(Finding::error("RENDER", err.to_string()).on_page(path)),
        Err(join_error) => {
            return failure(Finding::error("RENDER", join_error.to_string()).on_page(path));
        }
    };

    let links = extract_links(&page, &url);
    let findings = rules::run_all(page, Arc::clone(&shared.options))
        .await
        .into_iter()
        .map(|finding| finding.on_page(path.clone()))
        .collect();
    PageScan { findings, links }
}

fn failure(finding: Finding) -> PageScan {
    PageScan {
        findings: vec![finding],
        links: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut pages: Vec<&str> = findings.iter().filter_map(|f| f.page.as_deref()).collect();
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
    async fn start_page_errors_are_reported_too() {
        let server = MockServer::start().await;
        serve(&server, "/", ResponseTemplate::new(503), 1).await;

        let findings = run(&server, None).await;
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].page.as_deref(), Some("/"));
    }
}
