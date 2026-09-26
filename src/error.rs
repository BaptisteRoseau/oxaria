use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckerError {
    #[error("Failed to fetch URL: {0}")]
    HttpFetch(#[from] reqwest::Error),

    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to render HTML")]
    Render,
}
