mod cli;
mod error;
mod logging;
mod page;
mod report;
mod rules;
mod site;

use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use tracing::warn;
use url::Url;

use cli::CliConfig;
use error::CheckerError;
use rules::{CheckOptions, Finding};

#[tokio::main]
async fn main() -> ExitCode {
    let config = CliConfig::parse();
    logging::init_logger(config.verbose);

    match check(&config).await {
        Ok(findings) => {
            report::print_findings(&findings);
            report::exit_code(&findings)
        }
        Err(err) => {
            tracing::error!("{err}");
            ExitCode::from(1)
        }
    }
}

async fn check(config: &CliConfig) -> Result<Vec<Finding>, CheckerError> {
    let options = Arc::new(CheckOptions::from(config));
    match (config.full_site_scan, page::is_url(&config.path_or_url)) {
        (true, true) => {
            let start = Url::parse(&config.path_or_url)?;
            site::crawl(start, options, config.full_site_scan_max_pages).await
        }
        (true, false) => {
            warn!("--full-site-scan only applies to URLs; scanning the single file");
            check_single_page(&config.path_or_url, options).await
        }
        (false, _) => check_single_page(&config.path_or_url, options).await,
    }
}

async fn check_single_page(
    path_or_url: &str,
    options: Arc<CheckOptions>,
) -> Result<Vec<Finding>, CheckerError> {
    let html = page::load_html(path_or_url).await?;
    let page = Arc::new(page::RenderedPage {
        url: page_url(path_or_url),
        ..page::render(&html)?
    });
    Ok(rules::run_all(page, options).await)
}

/// Only for real URLs: `Url::parse` would also accept a Windows path like
/// `C:\page.html` as a URL with scheme `c`.
fn page_url(path_or_url: &str) -> Option<Url> {
    page::is_url(path_or_url)
        .then(|| Url::parse(path_or_url).ok())
        .flatten()
}
