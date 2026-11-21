//! Portal cancellation supersedes old intent without banning a newly reviewed move.

use axum::http::StatusCode;
use ciproxy_store_interface::billing::payments::Settlement;
use serde_json::json;

use super::credit_test_support::{business, pay, priced_requests, review, upgrade, work};
use super::fake_subscriptions::SubscriptionOutcome;
use super::pending_credit_tests::machine_credit;
use super::support::{Dashboard, at, json, signature};

#[tokio::test]
async fn portal_cancellation_supersedes_unknown_intent_but_allows_a_new_scheduled_quote() {
    let (mut dashboard, payer) = machine_credit().await;
    let quote = review(&dashboard, &payer).await;
    dashboard
        .fake
        .expect_subscription(upgrade(1795089600, -1000), SubscriptionOutcome::Reject(503));
    assert_eq!(
        pay(&dashboard, &payer, &quote).await.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        remote.subscription.cancel_at_period_end = true;
        remote.credits.as_mut().unwrap().invoice_reads_fail = true;
    }
    subscription_event(&dashboard, "evt_portal_cancel").await;
    assert_eq!(work(&dashboard, &quote).settlement, Settlement::Unresolved);
    dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .subscription
        .cancel_at_period_end = false;
    dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .credits
        .as_mut()
        .unwrap()
        .invoice_reads_fail = false;
    subscription_event(&dashboard, "evt_portal_undo").await;
    dashboard.restart().await;
    assert_eq!(
        pay(&dashboard, &payer, &quote).await.status(),
        StatusCode::CONFLICT
    );
    assert_eq!(priced_requests(&dashboard), 1);
    let held = work(&dashboard, &quote);
    assert_eq!(held.settlement, Settlement::Unresolved);
    assert_eq!(held.restoration, None);
    dashboard.close().await;

    let (dashboard, payer) = machine_credit().await;
    dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .subscription
        .cancel_at_period_end = true;
    let fresh = review(&dashboard, &payer).await;
    dashboard.fake.expect_subscription(
        upgrade(1795089600, -1000),
        SubscriptionOutcome::respond(business(&dashboard)),
    );
    let response = json(pay(&dashboard, &payer, &fresh).await, StatusCode::OK).await;
    assert_eq!(response["state"], "confirmed");
    assert_eq!(work(&dashboard, &fresh).settlement, Settlement::Consumed);
    assert_eq!(priced_requests(&dashboard), 1);
    let state = dashboard
        .fixture
        .database
        .store
        .billing(dashboard.fixture.workspace)
        .unwrap()
        .state;
    assert_eq!(state.cancel_at, Some(at("2026-12-04T12:00:00Z")));
    dashboard.close().await;
}

async fn subscription_event(dashboard: &Dashboard, id: &str) {
    let raw = serde_json::to_vec(&json!({"id":id,"created":1795089600,
        "type":"customer.subscription.updated","data":{"object":{"id":"sub_recorded"}}}))
    .unwrap();
    let response = dashboard
        .client
        .post(format!("{}/hooks/stripe", dashboard.origin))
        .header(
            "Stripe-Signature",
            signature(&raw, at("2026-11-19T12:00:00Z")),
        )
        .body(raw)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
