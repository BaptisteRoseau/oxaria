mod cli;
mod error;
mod logging;
mod page;
mod report;
mod rules;

use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;

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
    let html = page::load_html(&config.path_or_url).await?;
    let page = Arc::new(page::render(&html)?);
    let options = Arc::new(CheckOptions::from(config));
    Ok(rules::run_all(page, options).await)
}
