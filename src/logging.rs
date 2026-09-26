use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Initializes a global tracing subscriber. `--verbose` raises the level to include `info`.
pub fn init_logger(verbose: bool) {
    let level = if verbose { Level::INFO } else { Level::WARN };

    FmtSubscriber::builder()
        .with_max_level(level)
        .without_time()
        .with_level(true)
        .with_target(false)
        .init();
}
