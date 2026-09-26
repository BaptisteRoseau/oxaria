use tracing::level_filters::LevelFilter;
use tracing_subscriber::FmtSubscriber;

/// Initializes a global tracing subscriber. `--verbose` raises the level to include `info`;
/// `--quiet` turns logging off entirely, since it writes to stdout.
pub fn init_logger(verbose: bool, quiet: bool) {
    let level = match (quiet, verbose) {
        (true, _) => LevelFilter::OFF,
        (false, true) => LevelFilter::INFO,
        (false, false) => LevelFilter::WARN,
    };

    FmtSubscriber::builder()
        .with_max_level(level)
        .without_time()
        .with_level(true)
        .with_target(false)
        .init();
}
