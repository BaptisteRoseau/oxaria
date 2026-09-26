//! Resolves the CLI's single `path_or_url` argument into raw HTML text, fetching it over HTTP(S)
//! when it looks like a URL and reading it from disk otherwise.

use tracing::info;

use crate::error::CheckerError;

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
    let response = reqwest::get(url).await?.error_for_status()?;
    Ok(response.text().await?)
}

async fn read_file(path: &str) -> Result<String, CheckerError> {
    info!("Loading file {path}");
    Ok(tokio::fs::read_to_string(path).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

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
