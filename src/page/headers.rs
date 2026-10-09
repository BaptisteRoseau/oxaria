//! The headers sent with every request: the built-in `Accept`/`User-Agent`
//! plus whatever `-H/--header` adds, overrides or removes, following curl's
//! syntax (`Name: value` sets, `Name:` removes, `Name;` sends it empty).

use std::collections::HashSet;

use reqwest::header::{ACCEPT, HeaderMap, HeaderName, HeaderValue, USER_AGENT};

use super::fetch::{ACCEPT_HTML, CHECKER_USER_AGENT};

/// What reqwest strips from a redirect that changes scheme, host or port
/// (`remove_sensitive_headers` in its `redirect.rs`).
const DROPPED_ON_ORIGIN_CHANGE: [&str; 5] = [
    "authorization",
    "cookie",
    "cookie2",
    "proxy-authorization",
    "www-authenticate",
];

/// One `-H/--header` argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderArg {
    Set(HeaderName, HeaderValue),
    Remove(HeaderName),
}

pub fn parse_header(arg: &str) -> Result<HeaderArg, String> {
    match arg.split_once(':') {
        Some((name, value)) => match value.trim() {
            "" => Ok(HeaderArg::Remove(header_name(name)?)),
            value => Ok(HeaderArg::Set(header_name(name)?, header_value(value)?)),
        },
        None => match arg.strip_suffix(';') {
            Some(name) => Ok(HeaderArg::Set(
                header_name(name)?,
                HeaderValue::from_static(""),
            )),
            None => Err(format!("expected \"Name: value\", got {arg:?}")),
        },
    }
}

fn header_name(name: &str) -> Result<HeaderName, String> {
    HeaderName::from_bytes(name.as_bytes()).map_err(|_| format!("invalid header name {name:?}"))
}

fn header_value(value: &str) -> Result<HeaderValue, String> {
    HeaderValue::from_str(value).map_err(|_| format!("invalid header value {value:?}"))
}

#[derive(Debug, Clone)]
pub struct RequestHeaders {
    headers: HeaderMap,
    custom: bool,
}

impl RequestHeaders {
    pub fn new(args: &[HeaderArg]) -> Self {
        let mut headers = built_in_headers();
        apply(&mut headers, args);
        RequestHeaders {
            headers,
            custom: !args.is_empty(),
        }
    }

    pub fn map(&self) -> &HeaderMap {
        &self.headers
    }

    /// Any `-H` header might be a secret and would follow a redirect, so
    /// with one, a redirect may change scheme or port but not host.
    pub fn pins_host(&self) -> bool {
        self.custom
    }

    /// Whether reqwest would drop some `-H` header on a scheme, host or
    /// port change.
    pub fn drops_on_origin_change(&self) -> bool {
        self.custom
            && DROPPED_ON_ORIGIN_CHANGE
                .iter()
                .any(|name| self.headers.contains_key(*name))
    }
}

impl Default for RequestHeaders {
    fn default() -> Self {
        RequestHeaders::new(&[])
    }
}

fn built_in_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_HTML));
    headers.insert(USER_AGENT, HeaderValue::from_static(CHECKER_USER_AGENT));
    headers
}

/// A name given with `-H` replaces the built-in value, while repeating it
/// sends every value, as curl does.
fn apply(headers: &mut HeaderMap, args: &[HeaderArg]) {
    let mut overridden = HashSet::new();
    for arg in args {
        match arg {
            HeaderArg::Remove(name) => {
                headers.remove(name);
            }
            HeaderArg::Set(name, value) => {
                if overridden.insert(name.clone()) {
                    headers.remove(name);
                }
                headers.append(name.clone(), value.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn set(name: &'static str, value: &'static str) -> HeaderArg {
        HeaderArg::Set(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        )
    }

    fn values<'a>(headers: &'a HeaderMap, name: &str) -> Vec<&'a str> {
        headers
            .get_all(name)
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect()
    }

    #[rstest]
    #[case("X-Token: abc", set("x-token", "abc"))]
    #[case("X-Token:abc", set("x-token", "abc"))]
    #[case("X-Token:   a b c  ", set("x-token", "a b c"))]
    #[case("Authorization: Bearer a:b", set("authorization", "Bearer a:b"))]
    #[case("X-Empty;", set("x-empty", ""))]
    #[case("User-Agent:", HeaderArg::Remove(USER_AGENT))]
    #[case("Accept:   ", HeaderArg::Remove(ACCEPT))]
    fn parses_curl_header_syntax(#[case] arg: &str, #[case] expected: HeaderArg) {
        assert_eq!(parse_header(arg), Ok(expected));
    }

    #[rstest]
    #[case("X-Token")]
    #[case(": value")]
    #[case("Bad Name: value")]
    #[case("X-Token: line\nbreak")]
    fn rejects_malformed_headers(#[case] arg: &str) {
        assert!(parse_header(arg).is_err());
    }

    #[test]
    fn built_in_headers_are_sent_by_default() {
        let headers = RequestHeaders::default();
        assert_eq!(values(headers.map(), "accept"), vec![ACCEPT_HTML]);
        assert_eq!(
            values(headers.map(), "user-agent"),
            vec![CHECKER_USER_AGENT]
        );
    }

    #[test]
    fn a_header_replaces_the_built_in_value() {
        let headers = RequestHeaders::new(&[set("user-agent", "custom/1.0")]);
        assert_eq!(values(headers.map(), "user-agent"), vec!["custom/1.0"]);
    }

    #[test]
    fn a_repeated_header_sends_every_value() {
        let args = [set("x-tag", "a"), set("x-tag", "b")];
        let headers = RequestHeaders::new(&args);
        assert_eq!(values(headers.map(), "x-tag"), vec!["a", "b"]);
    }

    #[test]
    fn an_empty_value_removes_the_header() {
        let headers = RequestHeaders::new(&[HeaderArg::Remove(ACCEPT)]);
        assert!(headers.map().get(ACCEPT).is_none());
    }

    #[rstest]
    #[case(&["User-Agent: custom/1.0"], false)]
    #[case(&["Authorization: Bearer t", "X-Tag: a"], true)]
    #[case(&["X-Tag: a", "Cookie: a=1"], true)]
    fn knows_when_reqwest_drops_a_header(#[case] args: &[&str], #[case] expected: bool) {
        let args: Vec<HeaderArg> = args.iter().map(|arg| parse_header(arg).unwrap()).collect();
        assert_eq!(
            RequestHeaders::new(&args).drops_on_origin_change(),
            expected
        );
    }

    #[rstest]
    #[case(&[], false)]
    #[case(&["User-Agent:"], true)]
    #[case(&["X-Token: abc"], true)]
    fn any_header_pins_the_host(#[case] args: &[&str], #[case] expected: bool) {
        let args: Vec<HeaderArg> = args.iter().map(|arg| parse_header(arg).unwrap()).collect();
        assert_eq!(RequestHeaders::new(&args).pins_host(), expected);
    }
}
