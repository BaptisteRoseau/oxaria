use url::Url;

use super::Standard;
use crate::page::ElementRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule_id: &'static str,
    /// Set by the registry for every rule finding; `None` for failures of
    /// the checker itself (`FETCH`, `HTTP`, `SCAN`, ...).
    pub standard: Option<Standard>,
    pub severity: Severity,
    pub message: String,
    /// Selector-like path of the element the finding is about, for rules
    /// that point at one element (not page-level or stylesheet rules).
    pub element: Option<String>,
    /// How to fix the issue, rustc-style.
    pub help: Option<String>,
    /// URL of the page this finding belongs to, as it was actually served
    /// (after redirects); only set during a full-site scan, where a single
    /// report covers many pages.
    pub page: Option<Url>,
}

impl Finding {
    pub(crate) fn error(rule_id: &'static str, message: String) -> Self {
        Finding {
            rule_id,
            standard: None,
            severity: Severity::Error,
            message,
            element: None,
            help: None,
            page: None,
        }
    }

    pub(super) fn warning(rule_id: &'static str, message: String) -> Self {
        Finding {
            severity: Severity::Warning,
            ..Finding::error(rule_id, message)
        }
    }

    pub(super) fn at(self, element: ElementRef) -> Self {
        Finding {
            element: Some(element.selector()),
            ..self
        }
    }

    pub(super) fn help(self, help: impl Into<String>) -> Self {
        Finding {
            help: Some(help.into()),
            ..self
        }
    }

    pub(crate) fn in_standard(self, standard: Standard) -> Self {
        Finding {
            standard: Some(standard),
            ..self
        }
    }

    /// Query and fragment are dropped: the crawl treats URLs differing only
    /// by them as the same page.
    pub fn on_page(self, page: &Url) -> Self {
        let mut page = page.clone();
        page.set_query(None);
        page.set_fragment(None);
        Finding {
            page: Some(page),
            ..self
        }
    }

    #[cfg(test)]
    pub fn page_path(&self) -> Option<&str> {
        self.page.as_ref().map(Url::path)
    }
}
