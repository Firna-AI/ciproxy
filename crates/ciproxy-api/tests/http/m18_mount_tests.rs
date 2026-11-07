//! Supported Stripe Checkout delivery through a started dashboard server.

use axum::http::StatusCode;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::{Value, json};
use tokio::net::TcpListener;

use super::m18_stripe::{Scenario, Stripe};
use super::m18_support::{Pages, signature};

#[tokio::test]
async fn m18_bound_server_supports_checkout_on_the_dashboard_host() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider_for(Scenario::Expired));
    stripe.bind(pages.fixture.workspace);
    let billing = pages.fixture.login(7, Some(Role::Billing)).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let application = pages.application();
    let server = tokio::spawn(async move { axum::serve(listener, application).await.unwrap() });
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let cookie = format!("__Host-ciproxy={}", billing.cookie);
    let response = client
        .get(format!("{origin}/api{}", pages.fixture.path("/billing")))
        .header("Cookie", &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-ciproxy-build"], "http-test-build");
    let response = client
        .post(format!("{origin}/api{}", pages.fixture.path("/checkout")))
        .header("Cookie", &cookie)
        .header("Origin", &pages.fixture.state.config.public_origin)
        .header("x-ciproxy-csrf", &billing.csrf)
        .json(&json!({"tier":"team","add_ons":{"start_blocks":0,"machines":0}}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.json::<Value>().await.unwrap()["state"], "open");
    stripe.complete(false);
    let raw = br#"{ "id":"evt_mount_m18", "created":1793966400, "type":"checkout.session.completed", "data":{"object":{"id":"cs_m18"}} }"#;
    let response = client
        .post(format!("{origin}/hooks/stripe"))
        .header("Stripe-Signature", signature(raw))
        .body(raw.as_slice())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["Cache-Control"], "no-store");
    assert_eq!(response.json::<String>().await.unwrap(), "applied");
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
