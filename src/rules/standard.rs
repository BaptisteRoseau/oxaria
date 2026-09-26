use std::fmt;

use super::{act, aria12, html_aria, wcag22};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Standard {
    Wcag22,
    Aria12,
    HtmlAria,
    Act,
}

impl Standard {
    pub fn name(self) -> &'static str {
        match self {
            Standard::Wcag22 => "WCAG 2.2",
            Standard::Aria12 => "WAI-ARIA 1.2",
            Standard::HtmlAria => "ARIA in HTML",
            Standard::Act => "ACT",
        }
    }

    /// The W3C page documenting one of this standard's rules.
    pub fn reference_url(self, rule_id: &str) -> Option<String> {
        match self {
            Standard::Wcag22 => wcag22::reference_url(rule_id),
            Standard::Aria12 => Some(aria12::reference_url(rule_id)),
            Standard::HtmlAria => Some(html_aria::reference_url(rule_id)),
            Standard::Act => Some(act::reference_url(rule_id)),
        }
    }
}

impl fmt::Display for Standard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Standard::Wcag22, "WCAG 2.2")]
    #[case(Standard::Aria12, "WAI-ARIA 1.2")]
    #[case(Standard::HtmlAria, "ARIA in HTML")]
    #[case(Standard::Act, "ACT")]
    fn display_name(#[case] standard: Standard, #[case] expected: &str) {
        assert_eq!(standard.to_string(), expected);
    }

    #[rstest]
    #[case(
        Standard::Wcag22,
        "H44",
        "https://www.w3.org/WAI/WCAG22/Techniques/html/H44"
    )]
    #[case(
        Standard::Act,
        "2779a5",
        "https://www.w3.org/WAI/standards-guidelines/act/rules/2779a5/"
    )]
    #[case(
        Standard::Aria12,
        "ARIA-UNKNOWN",
        "https://www.w3.org/TR/wai-aria-1.2/"
    )]
    #[case(Standard::HtmlAria, "HTMLARIA999", "https://www.w3.org/TR/html-aria/")]
    fn reference_url_dispatches_on_the_standard(
        #[case] standard: Standard,
        #[case] rule_id: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(standard.reference_url(rule_id).as_deref(), Some(expected));
    }
}
