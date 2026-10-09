use thiserror::Error;
use url::Url;

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

    #[error(transparent)]
    LeakingRedirect(#[from] Box<LeakingRedirect>),
}

/// Raised instead of following a redirect while `-H` headers are set, when
/// following it would expose them.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{from} redirects to {to}, {risk}")]
pub struct LeakingRedirect {
    pub from: Url,
    pub to: Url,
    pub risk: RedirectRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectRisk {
    OtherHost,
    Downgrade,
    /// reqwest strips credential headers on a scheme, host or port change.
    DroppedHeaders,
}

impl std::fmt::Display for RedirectRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RedirectRisk::OtherHost => {
                "which would leak your headers to another host; re-run on it directly if that is \
                 what you intended"
            }
            RedirectRisk::Downgrade => "which would send your headers unencrypted over http",
            RedirectRisk::DroppedHeaders => {
                "where your headers would not be sent; re-run on it directly instead"
            }
        })
    }
}

impl From<LeakingRedirect> for CheckerError {
    fn from(refused: LeakingRedirect) -> Self {
        CheckerError::LeakingRedirect(Box::new(refused))
    }
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
            CheckerError::LeakingRedirect(_) => "REDIRECT",
        }
    }
}
