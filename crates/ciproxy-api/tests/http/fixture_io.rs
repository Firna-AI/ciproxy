//! HTTP response decoding shared by the real-store integration tests.

use axum::body::Body;
use axum::http::{Response, header};
use http_body_util::BodyExt;
use serde_json::Value;

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
