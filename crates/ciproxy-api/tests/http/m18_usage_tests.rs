//! Usage forecasts, advisory offers, plan cards, and banner priority over HTTP.

use axum::http::StatusCode;
use ciproxy_api::runtime::Runtime;
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};
use ciproxy_store_interface::dashboard::records::Role;
use diesel::prelude::*;
use serde_json::{Value, json};
use uuid::Uuid;

use super::fixture_io;
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};

#[tokio::test]
async fn m18_usage_forecasts_and_offers_remain_advisory_and_seed_checkout() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider());
    stripe.bind(pages.fixture.workspace);
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    pages.starts(1500);
    let response = pages.request(Some(&admin), "GET", "/billing", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let usage = fixture_io::json(response).await;
    assert_eq!(usage["starts"], "1500");
    assert_eq!(usage["expected_starts"], "7500");
    assert_eq!(usage["crossing_day"], "2026-11-09");
    assert_eq!(usage["banner"], Value::Null);
    assert_eq!(usage["status"]["period_start"], "2026-11-01T00:00:00Z");
    assert_eq!(usage["status"]["period_end"], "2026-12-01T00:00:00Z");
    pages.starts(2001);
    let over = fixture_io::json(pages.request(Some(&admin), "GET", "/billing", None).await).await;
    assert_eq!(over["starts_over"], "1");
    assert_eq!(over["banner"]["kind"], "soft_limit");
    assert_eq!(over["offer"]["selection"]["tier"], "team");
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
            .fixture
            .database
            .store
            .billing(pages.fixture.workspace)
            .unwrap()
            .state
            .customer_id,
        None
    );
    pages.starts(7000);
    let plans = fixture_io::json(pages.request(Some(&admin), "GET", "/plans", None).await).await;
    assert_eq!(plans["plans"][0]["current"], true);
    assert_eq!(plans["plans"][0]["can_choose"], false);
    assert_eq!(plans["plans"][1]["covers_use"], true);
    assert_eq!(plans["plans"][1]["initial_add_ons"]["start_blocks"], 1);
    assert_eq!(plans["plans"][2]["history_days"], 90);
    let later: Vec<_> = plans["plans"][2]["features"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|feature| feature["coming_later"] == true)
        .map(|feature| feature["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        later,
        [
            "Email and Slack alerts",
            "Single sign-on",
            "Audit log",
            "Custom VM images"
        ]
    );
    let review = fixture_io::json(
        pages
            .request(Some(&admin), "GET", "/checkout/review?tier=team", None)
            .await,
    )
    .await;
    assert_eq!(review["choice"]["selection"]["add_ons"]["start_blocks"], 1);
    assert_eq!(review["today_cents"], "5900");
    assert_eq!(review["billing_day"], 6);
    assert_eq!(review["prorated"], false);
    assert_eq!(
        pages
            .request(Some(&admin), "GET", "/checkout/review?tier=free", None)
            .await
            .status(),
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn m18_payment_banners_override_limits_and_cancellation_is_not_a_payment() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider());
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    pages.starts(2001);
    let response = pages.request(Some(&viewer), "GET", "/billing", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let free = fixture_io::json(response).await;
    assert_eq!(free["banner"]["action"], "ask_admin_or_billing");
    {
        let mut connection = PgConnection::establish(&pages.fixture.database.url).unwrap();
        diesel::sql_query("UPDATE organization_activation SET admitted=false,billing_hold=true,admin_approval_hold=true WHERE organization_id=$1")
            .bind::<diesel::sql_types::Uuid, _>(pages.fixture.organization.id).execute(&mut connection).unwrap();
    }
    let general =
        fixture_io::json(pages.request(Some(&viewer), "GET", "/banner", None).await).await;
    assert_eq!(general["banner"]["kind"], "soft_limit");
    let scoped = fixture_io::json(
        pages
            .request(Some(&viewer), "GET", "/banner?page=organizations", None)
            .await,
    )
    .await;
    assert_eq!(scoped["banner"]["kind"], "inactive_organizations");
    assert_eq!(
        scoped["banner"]["organizations"][0]["activation"]["reasons"],
        json!(["billing_upgrade", "admin_approval"])
    );
    assert_eq!(
        scoped["banner"]["organizations"][0]["activation"]["ask_admin"],
        true
    );
    let mut failed = pages.state(Tier::Team);
    failed.status = SubscriptionState::PaymentFailed;
    failed.failed_at = Some(at("2026-11-05T12:00:00Z"));
    failed.next_retry_at = Some(at("2026-11-08T12:00:00Z"));
    pages.save(&failed);
    let expected = json!({"kind":"payment_failed","failed_at":"2026-11-05T12:00:00Z","next_retry_at":"2026-11-08T12:00:00Z","action":"ask_admin_or_billing"});
    for route in [
        "/billing",
        "/banner",
        "/savings",
        "/jobs/live",
        "/jobs/history",
    ] {
        let body = fixture_io::json(pages.request(Some(&viewer), "GET", route, None).await).await;
        assert_eq!(body["banner"], expected, "{route}");
    }
    let plans = fixture_io::json(pages.request(Some(&viewer), "GET", "/plans", None).await).await;
    assert_eq!(plans["usage"]["banner"], expected);
    let mut ended = pages.state(Tier::Free);
    ended.status = SubscriptionState::Ended;
    ended.ended_banner_until = Some(at("2026-12-01T00:00:00Z"));
    pages.save(&ended);
    let banner = fixture_io::json(pages.request(Some(&viewer), "GET", "/banner", None).await).await;
    assert_eq!(banner["banner"]["kind"], "subscription_ended");
    ended.ended_banner_until = Some(at("2026-11-06T12:00:00Z"));
    pages.save(&ended);
    let banner = fixture_io::json(pages.request(Some(&viewer), "GET", "/banner", None).await).await;
    assert_eq!(banner["banner"]["kind"], "soft_limit");
    pages.save(&pages.state(Tier::Team));
    {
        let mut lock = pages
            .fixture
            .database
            .store
            .try_lock_billing(pages.fixture.workspace)
            .unwrap()
            .unwrap();
        lock.save_command(&BillingCommand {
            id: Uuid::new_v4(),
            started_at: pages.fixture.runtime.now(),
            payment_expires_at: None,
            kind: CommandKind::Cancel,
        })
        .unwrap();
    }
    let status =
        fixture_io::json(pages.request(Some(&viewer), "GET", "/billing", None).await).await;
    assert_eq!(status["status"]["payment_pending"], false);
    assert_eq!(status["status"]["operation_in_progress"], true);
}
