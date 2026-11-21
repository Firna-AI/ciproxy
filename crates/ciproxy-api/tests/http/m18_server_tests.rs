//! A real bound dashboard server proves API nesting and host-root Stripe delivery.

use axum::http::StatusCode;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::Value;
use tokio::net::TcpListener;

use super::m18_stripe::Stripe;
use super::m18_support::{Pages, signature};

#[tokio::test]
async fn m18_bound_server_serves_api_and_raw_host_root_webhook() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider());
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let application = pages.application();
    let server = tokio::spawn(async move { axum::serve(listener, application).await.unwrap() });
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let response = client
        .get(format!("{origin}/api{}", pages.fixture.path("/billing")))
        .header("Cookie", format!("__Host-ciproxy={}", viewer.cookie))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-ciproxy-build"], "http-test-build");
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["status"]["selection"]["tier"], "free");
    let raw = br#"{ "id":"evt_ignored_m18", "created":1793966400, "type":"charge.succeeded", "data":{"object":{"id":"ch_m18"}} }"#;
    let response = client
        .post(format!("{origin}/hooks/stripe"))
        .header("Stripe-Signature", signature(raw))
        .body(raw.as_slice())
        .send()
        .await
        .unwrap();
    let status = response.status();
    let body = response.text().await.unwrap();
    assert_eq!(status, StatusCode::OK, "synthetic response: {body}");
    assert_eq!(serde_json::from_str::<String>(&body).unwrap(), "ignored");
    assert_eq!(
        client
            .post(format!("{origin}/api/hooks/stripe"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    server.abort();
}
