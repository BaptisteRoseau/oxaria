//! Everything HTTP-related for a site crawl sits behind [`request`]: the
//! shared "already fetched" cache, rate limiting, retries on `429`/`503`,
//! following redirects only while they stay on the crawled host, and
//! classifying the final response as renderable HTML or something to ignore
//! (JSON, XML, images, ...).

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use reqwest::header::{CONTENT_TYPE, HeaderMap, LOCATION, RETRY_AFTER};
use reqwest::{Client, StatusCode, redirect};
use tokio::time::Instant;
use tracing::info;
use url::Url;

use crate::error::CheckerError;
use crate::page;

use super::links::visit_key;

const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRIES: u32 = 3;
const MAX_REDIRECTS: u32 = 10;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const RENDERABLE_CONTENT_TYPES: &[&str] = &["text/html", "application/xhtml+xml"];
/// `X-RateLimit-Reset` is epoch seconds on some servers (GitHub) and a
/// delta on others; anything this large can only be an epoch timestamp.
const EPOCH_SECONDS_THRESHOLD: u64 = 1_000_000_000;

#[derive(Debug)]
pub enum RequestOutcome {
    Html {
        final_url: Url,
        body: String,
    },
    /// The path, or the path of the redirect hop carried here, was already
    /// claimed by another request, so nothing more was downloaded.
    AlreadyFetched(Url),
    NotRenderable,
    OffDomainRedirect(Url),
    HttpError(StatusCode),
    Transport(reqwest::Error),
}

/// Which hosts a redirect may lead to before [`request`] gives up with
/// [`RequestOutcome::OffDomainRedirect`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectScope {
    /// Used for a crawl's start URL: `example.com` -> `www.example.com` is
    /// too common to refuse, and the host it lands on becomes the crawl's.
    AnyHost,
    SameHost(String),
}

impl RedirectScope {
    fn allows(&self, url: &Url) -> bool {
        match self {
            RedirectScope::AnyHost => true,
            RedirectScope::SameHost(host) => url.host_str() == Some(host.as_str()),
        }
    }
}

/// Shared by every crawl task, so a rate-limit signal on one response slows
/// down all of them and a path fetched by one is never fetched by another.
pub struct RequestContext {
    client: Client,
    next_allowed: Arc<RwLock<Instant>>,
    fetched: Arc<RwLock<HashSet<String>>>,
}

impl RequestContext {
    pub fn new() -> Result<Self, CheckerError> {
        // Redirects are followed by hand in `request` so each hop can be
        // checked against the redirect scope before it is requested.
        let client = Client::builder()
            .default_headers(page::request_headers())
            .redirect(redirect::Policy::none())
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(RequestContext {
            client,
            next_allowed: Arc::new(RwLock::new(Instant::now())),
            fetched: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    pub fn has_fetched(&self, url: &Url) -> bool {
        self.fetched
            .read()
            .expect("fetched set lock poisoned")
            .contains(&visit_key(url))
    }

    /// Check-and-insert under one write lock, so two tasks racing for the
    /// same path can't both see it as new.
    fn claim(&self, path: &str) -> bool {
        self.fetched
            .write()
            .expect("fetched set lock poisoned")
            .insert(path.to_string())
    }

    /// Re-reads after each sleep because another task may have pushed
    /// `next_allowed` further out (a new `429`) while this one was waiting.
    /// The lock is never held across the sleep, so waiting tasks don't
    /// block `defer`.
    async fn wait_turn(&self) {
        loop {
            let next_allowed = self.next_allowed_instant();
            if next_allowed <= Instant::now() {
                return;
            }
            tokio::time::sleep_until(next_allowed).await;
        }
    }

    fn next_allowed_instant(&self) -> Instant {
        *self.next_allowed.read().expect("rate limit lock poisoned")
    }

    fn defer(&self, delay: Duration) {
        let mut next_allowed = self.next_allowed.write().expect("rate limit lock poisoned");
        *next_allowed = (*next_allowed).max(Instant::now() + delay);
    }
}

pub async fn request(context: &RequestContext, url: &Url, scope: &RedirectScope) -> RequestOutcome {
    let mut current = url.clone();
    let mut claimed_path: Option<String> = None;
    let mut retries = 0;
    let mut redirects = 0;

    info!("Fetching {url}");
    loop {
        // Claimed per hop, not once up front: a redirect can land on a path
        // another task already fetched. Retries of the same path, and
        // redirects that only change the query string, keep their claim.
        let path = visit_key(&current);
        if claimed_path.as_deref() != Some(path.as_str()) {
            if !context.claim(&path) {
                return RequestOutcome::AlreadyFetched(current);
            }
            claimed_path = Some(path);
        }

        context.wait_turn().await;
        let response = match context.client.get(current.clone()).send().await {
            Ok(response) => response,
            Err(err) => return RequestOutcome::Transport(err),
        };
        let status = response.status();
        let headers = response.headers();

        if let Some(delay) = proactive_delay(headers) {
            context.defer(delay);
        }

        match status {
            _ if is_rate_limited(status, headers) && retries < MAX_RETRIES => {
                retries += 1;
                let delay = retry_after(headers);
                info!("{current} answered {status}; retrying in {delay:?}");
                context.defer(delay);
            }
            _ if status.is_redirection() => match redirect_target(&current, headers) {
                Some(target) if !scope.allows(&target) => {
                    return RequestOutcome::OffDomainRedirect(target);
                }
                Some(target) if redirects < MAX_REDIRECTS => {
                    info!("  {current} redirects to {target}");
                    redirects += 1;
                    current = target;
                }
                _ => return RequestOutcome::HttpError(status),
            },
            _ if status.is_client_error() || status.is_server_error() => {
                return RequestOutcome::HttpError(status);
            }
            _ if !is_renderable(headers) => return RequestOutcome::NotRenderable,
            _ => {
                return match response.text().await {
                    Ok(body) => RequestOutcome::Html {
                        final_url: current,
                        body,
                    },
                    Err(err) => RequestOutcome::Transport(err),
                };
            }
        }
    }
}

/// `503` is only treated as a rate limit when the server says when to come
/// back; without `Retry-After` it is an ordinary server error.
fn is_rate_limited(status: StatusCode, headers: &HeaderMap) -> bool {
    match status {
        StatusCode::TOO_MANY_REQUESTS => true,
        StatusCode::SERVICE_UNAVAILABLE => headers.contains_key(RETRY_AFTER),
        _ => false,
    }
}

fn redirect_target(current: &Url, headers: &HeaderMap) -> Option<Url> {
    let location = headers.get(LOCATION)?.to_str().ok()?;
    current.join(location).ok()
}

fn is_renderable(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(mime_essence)
        .is_some_and(|essence| RENDERABLE_CONTENT_TYPES.contains(&essence.as_str()))
}

fn mime_essence(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn retry_after(headers: &HeaderMap) -> Duration {
    headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_retry_after)
        .filter(|delay| !delay.is_zero())
        .unwrap_or(DEFAULT_RETRY_DELAY)
}

fn parse_retry_after(value: &str) -> Option<Duration> {
    match value.trim().parse::<u64>() {
        Ok(seconds) => Some(Duration::from_secs(seconds)),
        Err(_) => httpdate::parse_http_date(value.trim())
            .ok()
            .map(|date| date.duration_since(SystemTime::now()).unwrap_or_default()),
    }
}

/// Honors `RateLimit-*` / `X-RateLimit-*` headers on otherwise successful
/// responses: once the quota is exhausted, wait for the reset before the
/// next request instead of waiting to be told off with a `429`.
fn proactive_delay(headers: &HeaderMap) -> Option<Duration> {
    ["ratelimit", "x-ratelimit"]
        .iter()
        .find(|prefix| header_u64(headers, &format!("{prefix}-remaining")) == Some(0))
        .map(|prefix| {
            header_u64(headers, &format!("{prefix}-reset"))
                .map(reset_delay)
                .filter(|delay| !delay.is_zero())
                .unwrap_or(DEFAULT_RETRY_DELAY)
        })
}

fn header_u64(headers: &HeaderMap, name: &str) -> Option<u64> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

fn reset_delay(reset: u64) -> Duration {
    match reset >= EPOCH_SECONDS_THRESHOLD {
        true => (SystemTime::UNIX_EPOCH + Duration::from_secs(reset))
            .duration_since(SystemTime::now())
            .unwrap_or_default(),
        false => Duration::from_secs(reset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;
    use wiremock::matchers::{headers as header_values, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_str(value).unwrap());
        }
        map
    }

    fn html(body: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_raw(body.to_string(), "text/html; charset=utf-8")
    }

    fn localhost() -> RedirectScope {
        RedirectScope::SameHost("localhost".to_string())
    }

    fn localhost_context(server: &MockServer) -> (RequestContext, Url) {
        let base = Url::parse(&format!("http://localhost:{}/", server.address().port())).unwrap();
        (RequestContext::new().unwrap(), base)
    }

    #[test]
    fn retry_after_zero_defaults_to_one_second() {
        assert_eq!(
            retry_after(&headers(&[("retry-after", "0")])),
            DEFAULT_RETRY_DELAY
        );
    }

    #[test]
    fn retry_after_missing_defaults_to_one_second() {
        assert_eq!(retry_after(&HeaderMap::new()), DEFAULT_RETRY_DELAY);
    }

    #[test]
    fn retry_after_seconds_is_honored() {
        assert_eq!(
            retry_after(&headers(&[("retry-after", "5")])),
            Duration::from_secs(5)
        );
    }

    #[test]
    fn retry_after_http_date_is_the_remaining_time() {
        let date = httpdate::fmt_http_date(SystemTime::now() + Duration::from_secs(120));
        let delay = retry_after(&headers(&[("retry-after", &date)]));
        assert!(delay > Duration::from_secs(100) && delay <= Duration::from_secs(120));
    }

    #[test]
    fn retry_after_past_http_date_defaults_to_one_second() {
        let date = httpdate::fmt_http_date(SystemTime::UNIX_EPOCH);
        assert_eq!(
            retry_after(&headers(&[("retry-after", &date)])),
            DEFAULT_RETRY_DELAY
        );
    }

    #[test]
    fn proactive_delay_ignores_remaining_quota() {
        let map = headers(&[("x-ratelimit-remaining", "10"), ("x-ratelimit-reset", "30")]);
        assert_eq!(proactive_delay(&map), None);
    }

    #[test]
    fn proactive_delay_waits_for_delta_reset() {
        let map = headers(&[("ratelimit-remaining", "0"), ("ratelimit-reset", "7")]);
        assert_eq!(proactive_delay(&map), Some(Duration::from_secs(7)));
    }

    #[test]
    fn proactive_delay_waits_for_epoch_reset() {
        let reset = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60;
        let map = headers(&[
            ("x-ratelimit-remaining", "0"),
            ("x-ratelimit-reset", &reset.to_string()),
        ]);
        let delay = proactive_delay(&map).unwrap();
        assert!(delay > Duration::from_secs(50) && delay <= Duration::from_secs(60));
    }

    #[test]
    fn proactive_delay_without_reset_defaults_to_one_second() {
        let map = headers(&[("x-ratelimit-remaining", "0")]);
        assert_eq!(proactive_delay(&map), Some(DEFAULT_RETRY_DELAY));
    }

    #[test]
    fn html_content_types_are_renderable() {
        for content_type in [
            "text/html",
            "text/html; charset=utf-8",
            "TEXT/HTML",
            "application/xhtml+xml",
        ] {
            assert!(
                is_renderable(&headers(&[("content-type", content_type)])),
                "{content_type}"
            );
        }
    }

    #[test]
    fn api_and_binary_content_types_are_not_renderable() {
        for content_type in [
            "application/json",
            "application/xml",
            "text/xml",
            "image/png",
        ] {
            assert!(
                !is_renderable(&headers(&[("content-type", content_type)])),
                "{content_type}"
            );
        }
    }

    #[test]
    fn missing_content_type_is_not_renderable() {
        assert!(!is_renderable(&HeaderMap::new()));
    }

    #[test]
    fn rate_limit_statuses() {
        assert!(is_rate_limited(
            StatusCode::TOO_MANY_REQUESTS,
            &HeaderMap::new()
        ));
        assert!(is_rate_limited(
            StatusCode::SERVICE_UNAVAILABLE,
            &headers(&[("retry-after", "1")])
        ));
        assert!(!is_rate_limited(
            StatusCode::SERVICE_UNAVAILABLE,
            &HeaderMap::new()
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn deferring_delays_the_next_turn() {
        let context = RequestContext::new().unwrap();
        let start = Instant::now();
        context.defer(Duration::from_secs(3));
        context.wait_turn().await;
        assert!(start.elapsed() >= Duration::from_secs(3));
    }

    #[tokio::test(start_paused = true)]
    async fn waiting_task_honors_a_deferral_made_while_it_sleeps() {
        let context = Arc::new(RequestContext::new().unwrap());
        let start = Instant::now();
        context.defer(Duration::from_secs(1));
        let waiter = tokio::spawn({
            let context = Arc::clone(&context);
            async move { context.wait_turn().await }
        });
        tokio::time::sleep(Duration::from_millis(500)).await;
        context.defer(Duration::from_secs(5));
        waiter.await.unwrap();
        assert!(start.elapsed() >= Duration::from_millis(5500));
    }

    #[tokio::test]
    async fn request_returns_html_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/page"))
            .respond_with(html("<p>hi</p>"))
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        match request(&context, &base.join("/page").unwrap(), &localhost()).await {
            RequestOutcome::Html { body, .. } => assert_eq!(body, "<p>hi</p>"),
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    #[tokio::test]
    async fn request_asks_for_html_and_identifies_itself() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/negotiated"))
            // wiremock splits comma-separated header values before matching.
            .and(header_values(
                "accept",
                page::ACCEPT_HTML.split(',').collect(),
            ))
            .and(header_values("user-agent", vec![page::CHECKER_USER_AGENT]))
            .respond_with(html("<p>html</p>"))
            .with_priority(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/negotiated"))
            .respond_with(ResponseTemplate::new(400).set_body_raw("", "application/json"))
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        let outcome = request(&context, &base.join("/negotiated").unwrap(), &localhost()).await;
        assert!(
            matches!(outcome, RequestOutcome::Html { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn request_ignores_json() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api"))
            .respond_with(ResponseTemplate::new(200).set_body_raw("{}", "application/json"))
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        let outcome = request(&context, &base.join("/api").unwrap(), &localhost()).await;
        assert!(
            matches!(outcome, RequestOutcome::NotRenderable),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn request_reports_http_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        let outcome = request(&context, &base.join("/missing").unwrap(), &localhost()).await;
        assert!(
            matches!(outcome, RequestOutcome::HttpError(StatusCode::NOT_FOUND)),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn request_follows_same_domain_redirects() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/old"))
            .respond_with(ResponseTemplate::new(301).insert_header("location", "/new"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/new"))
            .respond_with(html("<p>new</p>"))
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        match request(&context, &base.join("/old").unwrap(), &localhost()).await {
            RequestOutcome::Html { final_url, .. } => assert_eq!(final_url.path(), "/new"),
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    #[tokio::test]
    async fn request_never_follows_off_domain_redirects() {
        let server = MockServer::start().await;
        let external = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/away"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", external.uri()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .respond_with(html("<p>elsewhere</p>"))
            .expect(0)
            .mount(&external)
            .await;
        let (context, base) = localhost_context(&server);

        let outcome = request(&context, &base.join("/away").unwrap(), &localhost()).await;
        assert!(
            matches!(outcome, RequestOutcome::OffDomainRedirect(_)),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn any_host_scope_follows_off_domain_redirects() {
        let server = MockServer::start().await;
        let external = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(301).insert_header("location", external.uri()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(html("<p>elsewhere</p>"))
            .expect(1)
            .mount(&external)
            .await;
        let (context, base) = localhost_context(&server);

        match request(&context, &base, &RedirectScope::AnyHost).await {
            RequestOutcome::Html { final_url, .. } => {
                assert_eq!(final_url.host_str(), Some("127.0.0.1"));
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    #[tokio::test]
    async fn request_never_fetches_a_path_twice() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/page"))
            .respond_with(html("<p>hi</p>"))
            .expect(1)
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        let first = request(&context, &base.join("/page?a=1").unwrap(), &localhost()).await;
        let second = request(&context, &base.join("/page?a=2").unwrap(), &localhost()).await;
        assert!(matches!(first, RequestOutcome::Html { .. }), "{first:?}");
        assert!(
            matches!(second, RequestOutcome::AlreadyFetched(_)),
            "{second:?}"
        );
        assert!(context.has_fetched(&base.join("/page").unwrap()));
    }

    #[tokio::test]
    async fn request_does_not_follow_redirects_to_fetched_paths() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(html("<p>home</p>"))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/alias"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", "/"))
            .expect(1)
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        request(&context, &base, &localhost()).await;
        let outcome = request(&context, &base.join("/alias").unwrap(), &localhost()).await;
        assert!(
            matches!(outcome, RequestOutcome::AlreadyFetched(_)),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn request_gives_up_on_redirect_loops() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/loop"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", "/loop"))
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        let outcome = request(&context, &base.join("/loop").unwrap(), &localhost()).await;
        assert!(
            matches!(outcome, RequestOutcome::HttpError(StatusCode::FOUND)),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn request_retries_after_rate_limit_with_one_second_default() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/slow"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
            .up_to_n_times(1)
            .with_priority(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/slow"))
            .respond_with(html("<p>ok</p>"))
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        let start = std::time::Instant::now();
        let outcome = request(&context, &base.join("/slow").unwrap(), &localhost()).await;
        assert!(
            matches!(outcome, RequestOutcome::Html { .. }),
            "{outcome:?}"
        );
        assert!(start.elapsed() >= DEFAULT_RETRY_DELAY);
    }

    #[tokio::test]
    async fn request_gives_up_after_max_retries() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/busy"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
            .expect(u64::from(MAX_RETRIES) + 1)
            .mount(&server)
            .await;
        let (context, base) = localhost_context(&server);

        let outcome = request(&context, &base.join("/busy").unwrap(), &localhost()).await;
        assert!(
            matches!(
                outcome,
                RequestOutcome::HttpError(StatusCode::TOO_MANY_REQUESTS)
            ),
            "{outcome:?}"
        );
    }
}
