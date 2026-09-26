use super::Report;

/// Renders a whole [`Report`] in one CI tool's format. Rendering to a
/// complete `String` (rather than streaming) lets the writer emit each
/// report in a single write, so reports sent to stdout in parallel can't
/// interleave mid-line.
pub trait Reporter: Send + Sync {
    fn render(&self, report: &Report) -> String;

    /// Whether the report is added to the end of an existing file instead
    /// of replacing it.
    fn appends(&self) -> bool {
        false
    }
}
