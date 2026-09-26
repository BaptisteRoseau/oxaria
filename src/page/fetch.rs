//! Resolves the CLI's single `path_or_url` argument into raw HTML text, fetching it over HTTP(S)
//! when it looks like a URL and reading it from disk otherwise.

use std::time::Duration;

use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, ClientBuilder};
use tracing::info;
use url::Url;

use crate::error::CheckerError;

/// Content-negotiating servers answer reqwest's default `Accept: */*` with
/// whatever they consider their primary format -- github.com/marketplace
/// replies `400` with an empty JSON body. Prefer HTML like a browser does,
/// but keep a `*/*` fallback so JSON/PDF-only URLs still answer `200` (and
/// get skipped as not renderable) instead of `406 Not Acceptable`.
/// `application/xml` is left out, unlike browsers, since it isn't rendered.
pub const ACCEPT_HTML: &str = "text/html,application/xhtml+xml;q=0.9,*/*;q=0.8";
/// reqwest sends no `User-Agent` at all by default, and some sites refuse
/// such requests outright -- crates.io answers `403`.
pub const CHECKER_USER_AGENT: &str = concat!("wcag-checker/", env!("CARGO_PKG_VERSION"));
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const RENDERABLE_CONTENT_TYPES: &[&str] = &["text/html", "application/xhtml+xml"];

/// Headers sent with every request, single-page and full-site alike.
fn request_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_HTML));
    headers.insert(USER_AGENT, HeaderValue::from_static(CHECKER_USER_AGENT));
    headers
}

pub fn client_builder() -> ClientBuilder {
    Client::builder()
        .default_headers(request_headers())
        .timeout(REQUEST_TIMEOUT)
}

pub fn is_renderable(headers: &HeaderMap) -> bool {
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

pub struct LoadedHtml {
    pub html: String,
    pub url: Option<Url>,
}

pub async fn load_html(path_or_url: &str) -> Result<LoadedHtml, CheckerError> {
    match is_url(path_or_url) {
        true => fetch_url(path_or_url).await,
        false => read_file(path_or_url).await,
    }
}

pub fn is_url(path_or_url: &str) -> bool {
    path_or_url.starts_with("http://") || path_or_url.starts_with("https://")
}

async fn fetch_url(url: &str) -> Result<LoadedHtml, CheckerError> {
    info!("Fetching {url}");
    let client = client_builder().build()?;
    let response = client.get(url).send().await?.error_for_status()?;
    let url = response.url().clone();
    if !is_renderable(response.headers()) {
        return Err(CheckerError::NotHtml(url.to_string()));
    }
    Ok(LoadedHtml {
        html: response.text().await?,
        url: Some(url),
    })
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
    use wiremock::matchers::{header, headers, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn is_url_accepts_http() {
        assert!(is_url("http://example.com/page.html"));
    }

    #[test]
    fn is_url_accepts_https() {
        assert!(is_url("https://example.com/page.html"));
    }

    #[test]
    fn is_url_rejects_local_path() {
        assert!(!is_url("tests/assets/clean.html"));
    }

    #[test]
    fn is_url_rejects_relative_path_with_colon_looking_segment() {
        assert!(!is_url("./page.html"));
    }

    #[tokio::test]
    async fn fetch_url_identifies_itself_and_asks_for_html() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(header("user-agent", CHECKER_USER_AGENT))
            // wiremock splits comma-separated header values before matching.
            .and(headers("accept", ACCEPT_HTML.split(',').collect()))
            .respond_with(ResponseTemplate::new(200).set_body_raw("<p>ok</p>", "text/html"))
            .mount(&server)
            .await;

        let loaded = fetch_url(&server.uri()).await.unwrap();
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
            .respond_with(ResponseTemplate::new(200).set_body_raw("<p>new</p>", "text/html"))
            .mount(&server)
            .await;

        let loaded = fetch_url(&format!("{}/old", server.uri())).await.unwrap();
        assert_eq!(loaded.url.unwrap().path(), "/new");
    }

    #[tokio::test]
    async fn fetch_url_rejects_non_html_responses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_raw("%PDF-1.7", "application/pdf"))
            .mount(&server)
            .await;

        let result = fetch_url(&server.uri()).await;
        assert!(matches!(result, Err(CheckerError::NotHtml(_))));
    }

    fn content_type(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(value).unwrap());
        headers
    }

    #[test]
    fn html_content_types_are_renderable() {
        for value in [
            "text/html",
            "text/html; charset=utf-8",
            "TEXT/HTML",
            "application/xhtml+xml",
        ] {
            assert!(is_renderable(&content_type(value)), "{value}");
        }
    }

    #[test]
    fn api_and_binary_content_types_are_not_renderable() {
        for value in [
            "application/json",
            "application/xml",
            "text/xml",
            "image/png",
        ] {
            assert!(!is_renderable(&content_type(value)), "{value}");
        }
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
