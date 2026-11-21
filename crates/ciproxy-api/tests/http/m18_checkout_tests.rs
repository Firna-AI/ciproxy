//! HTTP Checkout uniqueness, confirmation, signed raw events, and ownership.

use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ciproxy_billing::stripe::wire::types::CheckoutStatus;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;
use tower::ServiceExt;

use super::fixture_io;
use super::m18_stripe::{Scenario, Stripe};
use super::m18_support::{Pages, signature};

#[tokio::test]
async fn m18_checkout_reuses_one_payment_and_raw_signed_webhook_confirms_it() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider_for(Scenario::Checkout));
    stripe.bind(pages.fixture.workspace);
    let billing = pages.fixture.login(7, Some(Role::Billing)).await;
    let review = pages
        .request(
            Some(&billing),
            "GET",
            "/checkout/review?tier=team&start_blocks=0&machines=0",
            None,
        )
        .await;
    assert_eq!(review.status(), StatusCode::OK);
    let wanted = json!({"tier":"team","add_ons":{"start_blocks":0,"machines":0}});
    for _ in 0..2 {
        let response = pages
            .request(Some(&billing), "POST", "/checkout", Some(wanted.clone()))
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(fixture_io::json(response).await["state"], "open");
    }
    assert_eq!(stripe.checkouts.load(Ordering::SeqCst), 1);
    stripe.complete(false);
    let pending = fixture_io::json(
        pages
            .request(
                Some(&billing),
                "POST",
                "/checkout/return",
                Some(json!({"session_id":"cs_m18"})),
            )
            .await,
    )
    .await;
    assert_eq!(pending["outcome"]["state"], "confirming");
    let plans = fixture_io::json(pages.request(Some(&billing), "GET", "/plans", None).await).await;
    assert_eq!(plans["usage"]["status"]["payment_pending"], true);
    assert_eq!(plans["plans"][1]["can_choose"], false);
    assert_eq!(
        pages
            .fixture
            .database
            .store
            .billing(pages.fixture.workspace)
            .unwrap()
            .state
            .tier,
        Tier::Free
    );
    assert_eq!(
        pages
            .request(Some(&billing), "GET", "/checkout/review", None)
            .await
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        pages
            .request(
                Some(&billing),
                "POST",
                "/checkout/return",
                Some(json!({"session_id":"cs_foreign"}))
            )
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    stripe.complete(true);
    let raw = br#"{ "id":"evt_m18", "created":1793966400, "type":"checkout.session.completed", "data":{"object":{"id":"cs_m18"}} }"#;
    let send = |signature: String| {
        Request::builder()
            .method("POST")
            .uri("/hooks/stripe")
            .header("Stripe-Signature", signature)
            .body(Body::from(raw.as_slice()))
            .unwrap()
    };
    let response = pages
        .application()
        .oneshot(send("t=1,v1=00".into()))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    for expected in ["applied", "duplicate"] {
        let response = pages
            .application()
            .oneshot(send(signature(raw)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["Cache-Control"], "no-store");
        assert_eq!(fixture_io::json(response).await, expected);
    }
    let state = pages
        .fixture
        .database
        .store
        .billing(pages.fixture.workspace)
        .unwrap();
    assert_eq!(state.state.tier, Tier::Team);
    assert_eq!(state.command, None);
    let returned = fixture_io::json(
        pages
            .request(
                Some(&billing),
                "POST",
                "/checkout/return",
                Some(json!({"session_id":"cs_m18"})),
            )
            .await,
    )
    .await;
    assert_eq!(returned["outcome"]["state"], "confirmed");
}

#[tokio::test]
async fn m18_expired_checkout_preserves_free_and_can_be_tried_again() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider_for(Scenario::Expired));
    stripe.bind(pages.fixture.workspace);
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let review = pages
        .request(Some(&admin), "GET", "/checkout/review", None)
        .await;
    assert_eq!(review.status(), StatusCode::OK);
    let wanted = json!({"tier":"team","add_ons":{"start_blocks":0,"machines":0}});
    assert_eq!(
        pages
            .request(Some(&admin), "POST", "/checkout", Some(wanted.clone()))
            .await
            .status(),
        StatusCode::OK
    );
    stripe.session.lock().unwrap().status = CheckoutStatus::Expired;
    let expired = fixture_io::json(
        pages
            .request(
                Some(&admin),
                "POST",
                "/checkout/return",
                Some(json!({"session_id":"cs_m18"})),
            )
            .await,
    )
    .await;
    assert_eq!(expired["outcome"]["state"], "unfinished");
    assert_eq!(
        pages
            .fixture
            .database
            .store
            .billing(pages.fixture.workspace)
            .unwrap()
            .state
            .tier,
        Tier::Free
    );
    stripe.session.lock().unwrap().status = CheckoutStatus::Open;
    assert_eq!(
        pages
            .request(Some(&admin), "POST", "/checkout", Some(wanted))
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(stripe.checkouts.load(Ordering::SeqCst), 2);
}
