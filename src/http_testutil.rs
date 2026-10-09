//! Test-only wiremock fixtures shared by the HTTP unit tests.

use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub(crate) fn ok(body: impl Into<Vec<u8>>, content_type: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(body, content_type)
}

pub(crate) fn html(body: impl Into<Vec<u8>>) -> ResponseTemplate {
    ok(body, "text/html")
}

pub(crate) async fn serve(
    server: &MockServer,
    route: &str,
    response: ResponseTemplate,
    times: u64,
) {
    Mock::given(method("GET"))
        .and(path(route))
        .respond_with(response)
        .expect(times)
        .mount(server)
        .await;
}

/// Uses `localhost` so a second mock server reached via `127.0.0.1` is a
/// genuinely different host.
pub(crate) fn localhost_url(server: &MockServer, route: &str) -> Url {
    Url::parse(&format!(
        "http://localhost:{}{route}",
        server.address().port()
    ))
    .unwrap()
}
