//! Paid review identity, expiry, and durable retries through production HTTP.

use axum::http::StatusCode;
use ciproxy_store_interface::billing::state::Tier;
use reqwest::Method;

use super::fake_subscriptions::SubscriptionOutcome;
use super::quote_support::{advance, business, payment, review, reviewed, upgrade};
use super::support::json;

#[tokio::test]
async fn paid_quote_keeps_reviewed_charge_after_clock_movement_and_retry() {
    let (mut dashboard, payer, review) = reviewed().await;
    assert_eq!(review["today_cents"], "20000");
    advance(&dashboard, 1200);
    dashboard
        .fake
        .expect_subscription(upgrade(), SubscriptionOutcome::Reject(503));
    let failed = dashboard
        .request(
            Some(&payer),
            Method::POST,
            "/billing/change",
            Some(payment(&review)),
        )
        .await;
    assert_eq!(failed.status(), StatusCode::SERVICE_UNAVAILABLE);
    dashboard.restart().await;
    advance(&dashboard, 2400);
    dashboard.fake.expect_subscription(
        upgrade(),
        SubscriptionOutcome::respond(business(&dashboard)),
    );
    let paid = json(
        dashboard
            .request(
                Some(&payer),
                Method::POST,
                "/billing/change",
                Some(payment(&review)),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(paid["state"], "confirmed");
    let charges = dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .proration
        .as_ref()
        .unwrap()
        .charges
        .clone();
    assert_eq!(charges, vec![20000]);
    let posts = dashboard.fake.subscription_mutations();
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].key, posts[1].key);
    assert_eq!(posts[0].value("proration_date"), Some("1804334400"));
    assert_eq!(posts[0].form, posts[1].form);
    dashboard.close().await;
}

#[tokio::test]
async fn paid_quote_recovers_a_lost_success_once_after_restart() {
    let (mut dashboard, payer, reviewed) = reviewed().await;
    advance(&dashboard, 1200);
    dashboard.fake.state.lock().unwrap().lose_next_post = true;
    dashboard.fake.expect_subscription(
        upgrade(),
        SubscriptionOutcome::respond(business(&dashboard)),
    );
    assert_eq!(
        dashboard
            .request(
                Some(&payer),
                Method::POST,
                "/billing/change",
                Some(payment(&reviewed))
            )
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let before = dashboard
        .fixture
        .database
        .store
        .billing(dashboard.fixture.workspace)
        .unwrap();
    assert_eq!(before.state.tier, Tier::Team);
    assert_ne!(before.command, None);
    dashboard.restart().await;
    advance(&dashboard, 2400);
    let response = json(
        dashboard
            .request(
                Some(&payer),
                Method::POST,
                "/billing/change",
                Some(payment(&reviewed)),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(response["state"], "confirmed");
    let after = dashboard
        .fixture
        .database
        .store
        .billing(dashboard.fixture.workspace)
        .unwrap();
    assert_eq!(after.state.tier, Tier::Business);
    assert_eq!(after.command, None);
    let used = json(
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
    assert_eq!(used["code"], "review_required");
    assert_eq!(dashboard.fake.subscription_mutations().len(), 1);
    let charges = dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .proration
        .as_ref()
        .unwrap()
        .charges
        .clone();
    assert_eq!(charges, vec![20000]);
    dashboard.close().await;
}

#[tokio::test]
async fn paid_quote_starts_only_one_command_for_concurrent_submissions() {
    let (dashboard, payer, reviewed) = reviewed().await;
    dashboard.fake.expect_subscription(
        upgrade(),
        SubscriptionOutcome::respond(business(&dashboard)),
    );
    let (first, second) = tokio::join!(
        dashboard.request(
            Some(&payer),
            Method::POST,
            "/billing/change",
            Some(payment(&reviewed))
        ),
        dashboard.request(
            Some(&payer),
            Method::POST,
            "/billing/change",
            Some(payment(&reviewed))
        ),
    );
    let mut statuses = [first.status().as_u16(), second.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    assert_eq!(dashboard.fake.subscription_mutations().len(), 1);
    let quote = dashboard
        .fixture
        .database
        .store
        .try_lock_billing(dashboard.fixture.workspace)
        .unwrap()
        .unwrap()
        .quote(reviewed["quote"]["id"].as_str().unwrap().parse().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!((quote.monthly_cents, quote.today_cents), (31900, 20000));
    assert_ne!(quote.command_id, None);
    dashboard.close().await;
}

#[tokio::test]
async fn paid_quote_expiry_stops_at_period_end_and_another_quote_cannot_recover_payment() {
    let (dashboard, payer, first) = reviewed().await;
    let second = review(&dashboard, &payer).await;
    assert_ne!(first["quote"]["id"], second["quote"]["id"]);
    dashboard
        .fake
        .expect_subscription(upgrade(), SubscriptionOutcome::Reject(503));
    assert_eq!(
        dashboard
            .request(
                Some(&payer),
                Method::POST,
                "/billing/change",
                Some(payment(&first))
            )
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let response = dashboard
        .request(
            Some(&payer),
            Method::POST,
            "/billing/change",
            Some(payment(&second)),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(dashboard.fake.subscription_mutations().len(), 1);
    dashboard.save(&dashboard.paid());
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        for item in &mut remote.subscription.items.data {
            item.current_period_end = 1804334460;
        }
    }
    let near_end = review(&dashboard, &payer).await;
    assert_eq!(near_end["quote"]["expires_at"], "2027-03-06T12:01:00Z");
    advance(&dashboard, 60);
    let body = json(
        dashboard
            .request(
                Some(&payer),
                Method::POST,
                "/billing/change",
                Some(payment(&near_end)),
            )
            .await,
        StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(body["code"], "review_required");
    dashboard.close().await;
}
