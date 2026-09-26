use crate::cli::CliConfig;

pub const DEFAULT_CONTRAST_THRESHOLD: f64 = 4.5;
pub const DEFAULT_LARGE_TEXT_CONTRAST_THRESHOLD: f64 = 3.0;
pub const DEFAULT_TARGET_SIZE_THRESHOLD: f64 = 24.0;

pub struct CheckOptions {
    pub contrast_threshold: f64,
    pub large_text_contrast_threshold: f64,
    pub target_size_threshold: f64,
}

impl Default for CheckOptions {
    fn default() -> Self {
        CheckOptions {
            contrast_threshold: DEFAULT_CONTRAST_THRESHOLD,
            large_text_contrast_threshold: DEFAULT_LARGE_TEXT_CONTRAST_THRESHOLD,
            target_size_threshold: DEFAULT_TARGET_SIZE_THRESHOLD,
        }
    }
}

impl From<&CliConfig> for CheckOptions {
    fn from(config: &CliConfig) -> Self {
        CheckOptions {
            contrast_threshold: config.contrast_threshold,
            large_text_contrast_threshold: config.large_text_contrast_threshold,
            target_size_threshold: config.target_size_threshold,
        }
    }
}
