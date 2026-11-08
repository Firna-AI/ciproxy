//! Paid quote refusals through production HTTP and real PostgreSQL storage.

use axum::http::StatusCode;
use ciproxy_store_interface::workspaces::Workspace;
use reqwest::Method;
use serde_json::json;
use uuid::Uuid;

use super::fake_subscriptions::SubscriptionOutcome;
use super::quote_support::{advance, business, payment, reviewed, upgrade};
use super::support::json;

#[tokio::test]
async fn paid_quote_refuses_unknown_identity_without_a_stripe_mutation() {
    let (dashboard, payer, review) = reviewed().await;
    let mut body = payment(&review);
    body["quote_id"] = json!(Uuid::new_v4());
    let response = dashboard
        .request(Some(&payer), Method::POST, "/billing/change", Some(body))
        .await;
    let status = response.status();
    let body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::CONFLICT, json!("review_required"))
    );
    assert_eq!(
        dashboard.fake.posts("/v1/subscriptions/sub_recorded").len(),
        0
    );
    dashboard.close().await;
}

#[tokio::test]
async fn paid_quote_refuses_expired_review_before_starting_a_payment() {
    let (dashboard, payer, review) = reviewed().await;
    advance(&dashboard, 1800);
    let response = dashboard
        .request(
            Some(&payer),
            Method::POST,
            "/billing/change",
            Some(payment(&review)),
        )
        .await;
    let status = response.status();
    let body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::CONFLICT, json!("review_required"))
    );
    assert_eq!(
        dashboard.fake.posts("/v1/subscriptions/sub_recorded").len(),
        0
    );
    dashboard.close().await;
}

#[tokio::test]
async fn paid_quote_refuses_another_workspaces_review_without_disclosing_ownership() {
    let (dashboard, payer, reviewed) = reviewed().await;
    let other = Uuid::new_v4();
    dashboard
        .fixture
        .database
        .store
        .create_workspace(&Workspace {
            id: other,
            name: "Other payer".into(),
            delete_after: None,
        })
        .unwrap();
    let own_id = reviewed["quote"]["id"].as_str().unwrap().parse().unwrap();
    let mut foreign = dashboard
        .fixture
        .database
        .store
        .try_lock_billing(dashboard.fixture.workspace)
        .unwrap()
        .unwrap()
        .quote(own_id)
        .unwrap()
        .unwrap();
    foreign.payment.id = Uuid::new_v4();
    foreign.source.subscription_id = "sub_foreign".into();
    foreign.source.customer_id = "cus_foreign".into();
    dashboard
        .fixture
        .database
        .store
        .try_lock_billing(other)
        .unwrap()
        .unwrap()
        .save_quote(&foreign)
        .unwrap();
    let requests_before = dashboard.fake.state.lock().unwrap().requests.len();
    let response = dashboard
        .request(
            Some(&payer),
            Method::POST,
            "/billing/change",
            Some(json!({"quote_id":foreign.payment.id})),
        )
        .await;
    let body = json(response, StatusCode::CONFLICT).await;
    assert_eq!(body["code"], "review_required");
    let requests_after = dashboard.fake.state.lock().unwrap().requests.len();
    assert_eq!(requests_after, requests_before);
    assert_eq!(
        dashboard.fake.posts("/v1/subscriptions/sub_recorded").len(),
        0
    );
    dashboard.close().await;
}

#[tokio::test]
async fn paid_quote_cannot_be_used_again_after_completion_or_rejection() {
    for rejected in [false, true] {
        let (dashboard, payer, reviewed) = reviewed().await;
        let initial = dashboard.fake.state.lock().unwrap().subscription.clone();
        dashboard.fake.expect_subscription(
            upgrade(),
            if rejected {
                SubscriptionOutcome::Reject(400)
            } else {
                SubscriptionOutcome::respond(business(&dashboard))
            },
        );
        let response = dashboard
            .request(
                Some(&payer),
                Method::POST,
                "/billing/change",
                Some(payment(&reviewed)),
            )
            .await;
        assert_eq!(
            response.status(),
            if rejected {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::OK
            }
        );
        dashboard.fake.state.lock().unwrap().subscription = initial;
        dashboard.save(&dashboard.paid());
        let body = json(
            dashboard
                .request(
                    Some(&payer),
                    Method::POST,
                    "/billing/change",
                    Some(payment(&reviewed)),
                )
                .await,
            StatusCode::CONFLICT,
        )
        .await;
        assert_eq!(body["code"], "review_required", "rejected={rejected}");
        assert_eq!(
            dashboard.fake.posts("/v1/subscriptions/sub_recorded").len(),
            1
        );
        dashboard.close().await;
    }
}

#[tokio::test]
async fn paid_quote_requires_a_new_review_when_source_items_or_period_change() {
    for changed_period in [false, true] {
        let (dashboard, payer, reviewed) = reviewed().await;
        {
            let mut remote = dashboard.fake.state.lock().unwrap();
            if changed_period {
                for item in &mut remote.subscription.items.data {
                    item.current_period_start += 100_000;
                    item.current_period_end += 100_000;
                }
            } else {
                remote.subscription.items.data[0].id = "si_replaced_tier".into();
            }
        }
        let body = json(
            dashboard
                .request(
                    Some(&payer),
                    Method::POST,
                    "/billing/change",
                    Some(payment(&reviewed)),
                )
                .await,
            StatusCode::CONFLICT,
        )
        .await;
        assert_eq!(
            body["code"], "review_required",
            "changed_period={changed_period}"
        );
        assert_eq!(
            dashboard.fake.posts("/v1/subscriptions/sub_recorded").len(),
            0
        );
        dashboard.close().await;
    }
}
