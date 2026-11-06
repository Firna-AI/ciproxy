//! HTTP response decoding shared by the real-store integration tests.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;
use url::Url;

/// Observable result of a GitHub browser return.
#[derive(Clone, Copy)]
pub(super) enum CallbackOutcome {
    /// The browser receives a dashboard session and CSRF cookie.
    Accepted,
    /// The browser returns to sign-in without cookies.
    Refused,
}

pub(super) async fn callback(
    router: Router,
    request: Request<Body>,
    outcome: CallbackOutcome,
) -> Response<Body> {
    let response = router.oneshot(request).await.unwrap();
    assert_callback(&response, outcome);
    response
}

pub(super) fn assert_callback(response: &Response<Body>, outcome: CallbackOutcome) {
    let (location, sets_cookies) = match outcome {
        CallbackOutcome::Accepted => ("/", true),
        CallbackOutcome::Refused => ("/sign-in?reason=github", false),
    };
    let cookies: Vec<_> = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split_once('=').map(|(name, _)| name))
        .collect();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get("x-ciproxy-build").unwrap(),
        "http-test-build"
    );
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some(location)
    );
    assert_eq!(
        cookies.contains(&"__Host-ciproxy"),
        sets_cookies,
        "session cookie"
    );
    assert_eq!(
        cookies.contains(&"__Host-ciproxy-csrf"),
        sets_cookies,
        "CSRF cookie"
    );
    assert_eq!(
        response.headers().contains_key(header::SET_COOKIE),
        sets_cookies
    );
}

pub(super) fn cookie(response: &Response<Body>, name: &str) -> String {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(|value| {
            value
                .split(';')
                .next()
                .and_then(|value| value.split_once('='))
                .filter(|(key, _)| *key == name)
                .map(|(_, value)| value.to_owned())
        })
        .unwrap()
}

pub(super) async fn json(response: Response<Body>) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

pub(super) fn oauth_state(response: &Response<Body>) -> String {
    let location = Url::parse(
        response
            .headers()
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    location
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned()
}
