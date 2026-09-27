use std::num::NonZeroUsize;

use clap::Parser;

use crate::rules::{
    DEFAULT_CONTRAST_THRESHOLD, DEFAULT_LARGE_TEXT_CONTRAST_THRESHOLD,
    DEFAULT_TARGET_SIZE_THRESHOLD,
};

/// Check a web page against a subset of WCAG 2.2, WAI-ARIA 1.2, ARIA in HTML and ACT rules.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliConfig {
    /// HTML source: an http(s) URL, or a path to a local HTML file
    pub path_or_url: String,

    /// Also log info-level messages
    #[arg(short, long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Print nothing on stdout (findings, summary, and logs); only the exit code and the
    /// --report-* files remain
    #[arg(short, long)]
    pub quiet: bool,

    /// Minimum contrast ratio for normal-size text (WCAG 1.4.3 default: 4.5)
    #[arg(long, default_value_t = DEFAULT_CONTRAST_THRESHOLD)]
    pub contrast_threshold: f64,

    /// Minimum contrast ratio for large-scale text (WCAG 1.4.3 default: 3.0)
    #[arg(long, default_value_t = DEFAULT_LARGE_TEXT_CONTRAST_THRESHOLD)]
    pub large_text_contrast_threshold: f64,

    /// Minimum pointer target size in CSS pixels (WCAG 2.5.8 default: 24.0)
    #[arg(long, default_value_t = DEFAULT_TARGET_SIZE_THRESHOLD)]
    pub target_size_threshold: f64,

    /// When given a URL, also scan every same-domain page reachable through its links
    #[arg(long)]
    pub full_site_scan: bool,

    /// Maximum number of HTML pages checked during a full-site scan (default: no limit)
    #[arg(long, requires = "full_site_scan")]
    pub full_site_scan_max_pages: Option<NonZeroUsize>,

    /// Write a GitLab Code Quality report (artifacts:reports:codequality) to FILE ('-' for stdout)
    #[arg(long, value_name = "FILE")]
    pub report_gitlab: Option<String>,

    /// Append a GitHub Actions job summary (Markdown) to FILE ('-' for stdout); defaults to
    /// $GITHUB_STEP_SUMMARY, so it is written automatically inside GitHub Actions
    #[arg(long, value_name = "FILE", env = "GITHUB_STEP_SUMMARY")]
    pub report_github: Option<String>,

    /// Write a Jenkins Warnings NG report (recordIssues tool: issues()) to FILE ('-' for stdout)
    #[arg(long, value_name = "FILE")]
    pub report_jenkins: Option<String>,

    /// Write a JUnit XML report (Jenkins junit step, GitLab artifacts:reports:junit) to FILE ('-'
    /// for stdout)
    #[arg(long, value_name = "FILE")]
    pub report_junit: Option<String>,
}
