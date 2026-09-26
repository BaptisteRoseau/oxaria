use clap::Parser;

/// Check a web page against a subset of WCAG 2.2 rules.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliConfig {
    /// HTML source: an http(s) URL, or a path to a local HTML file
    pub path_or_url: String,

    /// Enable stdout output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Minimum contrast ratio for normal-size text (WCAG 1.4.3 default: 4.5)
    #[arg(long, default_value_t = 4.5)]
    pub contrast_threshold: f64,

    /// Minimum contrast ratio for large-scale text (WCAG 1.4.3 default: 3.0)
    #[arg(long, default_value_t = 3.0)]
    pub large_text_contrast_threshold: f64,

    /// Minimum pointer target size in CSS pixels (WCAG 2.5.8 default: 24.0)
    #[arg(long, default_value_t = 24.0)]
    pub target_size_threshold: f64,
}
