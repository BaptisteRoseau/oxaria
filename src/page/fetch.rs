//! Resolves the CLI's single `path_or_url` argument into raw HTML text, fetching it over HTTP(S)
//! when it looks like a URL and reading it from disk otherwise.

use std::time::Duration;

use reqwest::header::{CONTENT_TYPE, HeaderMap};
use reqwest::{Client, ClientBuilder, redirect};
use tracing::info;
use url::Url;

use crate::error::{CheckerError, LeakingRedirect, RedirectRisk};

use super::headers::RequestHeaders;

/// Content-negotiating servers answer reqwest's default `Accept: */*` with
/// whatever they consider their primary format -- github.com/marketplace
/// replies `400` with an empty JSON body. Prefer HTML like a browser does,
/// but keep a `*/*` fallback so JSON/PDF-only URLs still answer `200` (and
/// get skipped as not renderable) instead of `406 Not Acceptable`.
/// `application/xml` is left out, unlike browsers, since it isn't rendered.
pub const ACCEPT_HTML: &str = "text/html,application/xhtml+xml;q=0.9,*/*;q=0.8";
/// reqwest sends no `User-Agent` at all by default, and some sites refuse
/// such requests outright -- crates.io answers `403`.
pub const CHECKER_USER_AGENT: &str = concat!("oxaria/", env!("CARGO_PKG_VERSION"));
pub const MAX_REDIRECTS: usize = 10;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const RENDERABLE_CONTENT_TYPES: &[&str] = &["text/html", "application/xhtml+xml"];

pub fn client_builder(headers: &RequestHeaders) -> ClientBuilder {
    Client::builder()
        .default_headers(headers.map().clone())
        .timeout(REQUEST_TIMEOUT)
}

pub fn is_renderable(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(mime_essence)
        .is_some_and(|essence| {
            RENDERABLE_CONTENT_TYPES
                .iter()
                .any(|renderable| essence.eq_ignore_ascii_case(renderable))
        })
}

fn mime_essence(content_type: &str) -> &str {
    content_type.split(';').next().unwrap_or_default().trim()
}

pub struct LoadedHtml {
    pub html: String,
    pub url: Option<Url>,
}

pub async fn load_html(
    path_or_url: &str,
    headers: &RequestHeaders,
) -> Result<LoadedHtml, CheckerError> {
    match is_url(path_or_url) {
        true => fetch_url(path_or_url, headers).await,
        false => read_file(path_or_url).await,
    }
}

pub fn is_url(path_or_url: &str) -> bool {
    path_or_url.starts_with("http://") || path_or_url.starts_with("https://")
}

async fn fetch_url(url: &str, headers: &RequestHeaders) -> Result<LoadedHtml, CheckerError> {
    info!("Fetching {url}");
    let client = client_builder(headers)
        .redirect(redirect_policy(headers))
        .build()?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(refused_redirect_or_fetch)?
        .error_for_status()?;
    let url = response.url().clone();
    if !is_renderable(response.headers()) {
        return Err(CheckerError::NotHtml(url.to_string()));
    }
    Ok(LoadedHtml {
        html: response.text().await?,
        url: Some(url),
    })
}

fn redirect_policy(headers: &RequestHeaders) -> redirect::Policy {
    if !headers.pins_host() {
        return redirect::Policy::limited(MAX_REDIRECTS);
    }
    let headers = headers.clone();
    redirect::Policy::custom(move |attempt| {
        let previous = attempt.previous();
        let current = &previous[previous.len() - 1];
        let refused =
            redirect_refusal(&headers, current, attempt.url()).map(|risk| LeakingRedirect {
                from: previous[0].clone(),
                to: attempt.url().clone(),
                risk,
            });
        let too_many = previous.len() >= MAX_REDIRECTS;
        match (refused, too_many) {
            (Some(refused), _) => attempt.error(refused),
            (None, true) => attempt.error("too many redirects"),
            (None, false) => attempt.follow(),
        }
    })
}

/// The condition under which reqwest strips sensitive headers from a
/// redirect (`remove_sensitive_headers` in its `redirect.rs`).
fn changes_origin(current: &Url, target: &Url) -> bool {
    current.host_str() != target.host_str()
        || current.port_or_known_default() != target.port_or_known_default()
        || current.scheme() != target.scheme()
}

/// reqwest wraps the policy's error in its own, so it is dug back out.
fn refused_redirect_or_fetch(err: reqwest::Error) -> CheckerError {
    let refused = std::iter::successors(std::error::Error::source(&err), |source| source.source())
        .find_map(|source| source.downcast_ref::<LeakingRedirect>())
        .cloned();
    match refused {
        Some(refused) => refused.into(),
        None => err.into(),
    }
}

/// Why a redirect from `current` to `target` is refused, if it is: with
/// `-H` headers, a scheme or port change on the same host is fine, except
/// https -> http, or when a header that reqwest drops on such a change was
/// given. The full-site crawler keeps every header across redirects, but
/// applies the same rule so both modes behave alike.
pub fn redirect_refusal(
    headers: &RequestHeaders,
    current: &Url,
    target: &Url,
) -> Option<RedirectRisk> {
    if !headers.pins_host() {
        return None;
    }
    redirect_risk(current, target).or_else(|| {
        (headers.drops_on_origin_change() && changes_origin(current, target))
            .then_some(RedirectRisk::DroppedHeaders)
    })
}

fn redirect_risk(current: &Url, target: &Url) -> Option<RedirectRisk> {
    match (
        target.host_str() == current.host_str(),
        is_downgrade(current, target),
    ) {
        (false, _) => Some(RedirectRisk::OtherHost),
        (true, true) => Some(RedirectRisk::Downgrade),
        (true, false) => None,
    }
}

fn is_downgrade(current: &Url, target: &Url) -> bool {
    current.scheme() == "https" && target.scheme() == "http"
}

async fn read_file(path: &str) -> Result<LoadedHtml, CheckerError> {
    info!("Loading file {path}");
    Ok(LoadedHtml {
        html: tokio::fs::read_to_string(path).await?,
        url: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;
    use rstest::rstest;
    use wiremock::matchers::{header, headers, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::http_testutil::{html, localhost_url, ok};

    #[rstest]
    #[case("http://example.com/page.html", true)]
    #[case("https://example.com/page.html", true)]
    #[case("tests/assets/clean.html", false)]
    #[case("C:\\page.html", false)]
    fn is_url_accepts_only_http_and_https(#[case] path_or_url: &str, #[case] expected: bool) {
        assert_eq!(is_url(path_or_url), expected);
    }

    #[tokio::test]
    async fn fetch_url_identifies_itself_and_asks_for_html() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(header("user-agent", CHECKER_USER_AGENT))
            // wiremock splits comma-separated header values before matching.
            .and(headers("accept", ACCEPT_HTML.split(',').collect()))
            .respond_with(html("<p>ok</p>"))
            .mount(&server)
            .await;

        let loaded = fetch_url(&server.uri(), &RequestHeaders::default())
            .await
            .unwrap();
        assert_eq!(loaded.html, "<p>ok</p>");
    }

    fn custom_headers() -> RequestHeaders {
        RequestHeaders::new(&[crate::page::parse_header("Authorization: Bearer token").unwrap()])
    }

    async fn redirecting_server(location: &str) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", location))
            .mount(&server)
            .await;
        server
    }

    #[tokio::test]
    async fn fetch_url_refuses_a_redirect_to_another_host_with_headers() {
        let external = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(html("<p>ok</p>"))
            .expect(0)
            .mount(&external)
            .await;
        let server = redirecting_server(&external.uri()).await;

        let result = fetch_url(localhost_url(&server, "/").as_str(), &custom_headers()).await;
        match result {
            Err(CheckerError::LeakingRedirect(refused)) => {
                assert_eq!(refused.from, localhost_url(&server, "/"));
                assert_eq!(refused.to.host_str(), Some("127.0.0.1"));
                assert_eq!(refused.risk, RedirectRisk::OtherHost);
            }
            other => panic!("unexpected result: {:?}", other.map(|loaded| loaded.url)),
        }
    }

    fn url(url: &str) -> Url {
        Url::parse(url).unwrap()
    }

    #[rstest]
    #[case("http://google.com/", "https://google.com/", None)]
    #[case("http://google.com/", "http://google.com:8443/", None)]
    #[case("https://google.com/", "https://google.com:8443/a", None)]
    #[case(
        "http://google.com/",
        "http://calendar.google.com/",
        Some(RedirectRisk::OtherHost)
    )]
    #[case(
        "http://google.com/",
        "https://malicious.com/",
        Some(RedirectRisk::OtherHost)
    )]
    #[case(
        "https://google.com/",
        "http://google.com/",
        Some(RedirectRisk::Downgrade)
    )]
    #[case(
        "https://google.com/",
        "http://evil.com/",
        Some(RedirectRisk::OtherHost)
    )]
    fn only_same_host_redirects_without_downgrade_are_safe(
        #[case] current: &str,
        #[case] target: &str,
        #[case] expected: Option<RedirectRisk>,
    ) {
        assert_eq!(redirect_risk(&url(current), &url(target)), expected);
    }

    #[tokio::test]
    async fn fetch_url_refuses_a_port_change_that_would_drop_credentials() {
        let other_port = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(html("<p>ok</p>"))
            .expect(0)
            .mount(&other_port)
            .await;
        let server = redirecting_server(localhost_url(&other_port, "/").as_str()).await;

        let result = fetch_url(localhost_url(&server, "/").as_str(), &custom_headers()).await;
        match result {
            Err(CheckerError::LeakingRedirect(refused)) => {
                assert_eq!(refused.risk, RedirectRisk::DroppedHeaders)
            }
            other => panic!("unexpected result: {:?}", other.map(|loaded| loaded.url)),
        }
    }

    #[tokio::test]
    async fn fetch_url_follows_a_port_change_that_keeps_every_header() {
        let other_port = MockServer::start().await;
        Mock::given(method("GET"))
            .and(header("user-agent", "custom/1.0"))
            .respond_with(html("<p>ok</p>"))
            .expect(1)
            .mount(&other_port)
            .await;
        let server = redirecting_server(localhost_url(&other_port, "/").as_str()).await;
        let headers =
            RequestHeaders::new(&[crate::page::parse_header("User-Agent: custom/1.0").unwrap()]);

        let loaded = fetch_url(localhost_url(&server, "/").as_str(), &headers)
            .await
            .unwrap();
        assert_eq!(loaded.html, "<p>ok</p>");
    }

    #[rstest]
    #[case("http://google.com/", "http://google.com/a", false)]
    #[case("http://google.com/", "http://google.com:80/a", false)]
    #[case("http://google.com/", "https://google.com/", true)]
    #[case("http://google.com/", "http://google.com:8443/", true)]
    fn origin_changes_like_reqwest_sees_them(
        #[case] current: &str,
        #[case] target: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(changes_origin(&url(current), &url(target)), expected);
    }

    #[tokio::test]
    async fn fetch_url_follows_a_redirect_to_another_host_without_headers() {
        let external = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(html("<p>ok</p>"))
            .mount(&external)
            .await;
        let server = redirecting_server(&external.uri()).await;

        let loaded = fetch_url(
            localhost_url(&server, "/").as_str(),
            &RequestHeaders::default(),
        )
        .await
        .unwrap();
        assert_eq!(loaded.html, "<p>ok</p>");
    }

    #[tokio::test]
    async fn fetch_url_reports_the_url_after_redirects() {
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

        let loaded = fetch_url(&format!("{}/old", server.uri()), &RequestHeaders::default())
            .await
            .unwrap();
        assert_eq!(loaded.url.unwrap().path(), "/new");
    }

    #[tokio::test]
    async fn fetch_url_rejects_non_html_responses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ok("%PDF-1.7", "application/pdf"))
            .mount(&server)
            .await;

        let result = fetch_url(&server.uri(), &RequestHeaders::default()).await;
        assert!(matches!(result, Err(CheckerError::NotHtml(_))));
    }

    fn content_type(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(value).unwrap());
        headers
    }

    #[rstest]
    #[case("text/html", true)]
    #[case("text/html; charset=utf-8", true)]
    #[case("TEXT/HTML", true)]
    #[case("application/xhtml+xml", true)]
    #[case("application/json", false)]
    #[case("application/xml", false)]
    #[case("text/xml", false)]
    #[case("image/png", false)]
    fn only_html_content_types_are_renderable(#[case] value: &str, #[case] expected: bool) {
        assert_eq!(is_renderable(&content_type(value)), expected);
    }

    #[test]
    fn missing_content_type_is_not_renderable() {
        assert!(!is_renderable(&HeaderMap::new()));
    }

    #[tokio::test]
    async fn read_file_returns_contents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("page.html");
        tokio::fs::write(&path, "<html></html>").await.unwrap();
        let loaded = read_file(path.to_str().unwrap()).await.unwrap();
        assert_eq!(loaded.html, "<html></html>");
        assert_eq!(loaded.url, None);
    }

    #[tokio::test]
    async fn read_file_missing_file_errors() {
        let result = read_file("does/not/exist.html").await;
        assert!(result.is_err());
    }
}
