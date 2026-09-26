//! Resolves the CLI's single `path_or_url` argument into raw HTML text, fetching it over HTTP(S)
//! when it looks like a URL and reading it from disk otherwise.

use reqwest::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use tracing::info;

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

/// Headers sent with every request, single-page and full-site alike.
pub fn request_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_HTML));
    headers.insert(USER_AGENT, HeaderValue::from_static(CHECKER_USER_AGENT));
    headers
}

pub async fn load_html(path_or_url: &str) -> Result<String, CheckerError> {
    match is_url(path_or_url) {
        true => fetch_url(path_or_url).await,
        false => read_file(path_or_url).await,
    }
}

pub fn is_url(path_or_url: &str) -> bool {
    path_or_url.starts_with("http://") || path_or_url.starts_with("https://")
}

async fn fetch_url(url: &str) -> Result<String, CheckerError> {
    info!("Fetching {url}");
    let client = Client::builder()
        .default_headers(request_headers())
        .build()?;
    let response = client.get(url).send().await?.error_for_status()?;
    Ok(response.text().await?)
}

async fn read_file(path: &str) -> Result<String, CheckerError> {
    info!("Loading file {path}");
    Ok(tokio::fs::read_to_string(path).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, headers, method};
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
            .respond_with(ResponseTemplate::new(200).set_body_string("<p>ok</p>"))
            .mount(&server)
            .await;

        let body = fetch_url(&server.uri()).await.unwrap();
        assert_eq!(body, "<p>ok</p>");
    }

    #[tokio::test]
    async fn read_file_returns_contents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("page.html");
        tokio::fs::write(&path, "<html></html>").await.unwrap();
        let content = read_file(path.to_str().unwrap()).await.unwrap();
        assert_eq!(content, "<html></html>");
    }

    #[tokio::test]
    async fn read_file_missing_file_errors() {
        let result = read_file("does/not/exist.html").await;
        assert!(result.is_err());
    }
}
