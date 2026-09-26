use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckerError {
    #[error("Failed to fetch URL: {0}")]
    HttpFetch(#[from] reqwest::Error),

    #[error("Not an HTML page: {0}")]
    NotHtml(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to render HTML")]
    Render,
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("{first} and {second} both write to {destination}")]
    Conflict {
        first: &'static str,
        second: &'static str,
        destination: String,
    },

    #[error("{flag} would overwrite the checked file {path}")]
    OverwritesInput { flag: &'static str, path: String },

    #[error("{flag}: failed to write {destination}: {source}")]
    Write {
        flag: &'static str,
        destination: String,
        source: std::io::Error,
    },

    #[error("Report writer failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

impl CheckerError {
    /// The pseudo rule ID a fatal error is reported under, matching the IDs
    /// a full-site scan already uses for per-page failures.
    pub fn rule_id(&self) -> &'static str {
        match self {
            CheckerError::HttpFetch(_) | CheckerError::NotHtml(_) => "FETCH",
            CheckerError::InvalidUrl(_) | CheckerError::FileRead(_) => "INPUT",
            CheckerError::Render => "RENDER",
        }
    }
}
