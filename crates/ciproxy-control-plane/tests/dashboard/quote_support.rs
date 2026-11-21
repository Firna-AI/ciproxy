//! Reviewed paid changes with a request-driven Stripe proration calculation.

use axum::http::StatusCode;
use ciproxy_billing::catalog::PriceCode;
use ciproxy_billing::stripe::wire::types::Subscription;
use ciproxy_store_interface::dashboard::records::Role;
use reqwest::Method;
use serde_json::{Value, json};

use super::fake_requests::Proration;
use super::fake_subscriptions::{ChangeMode, ItemChange, SubscriptionRequest};
use super::fixture::Browser;
use super::support::{Dashboard, at, json};

pub(super) async fn reviewed() -> (Dashboard, Browser, Value) {
    let dashboard = Dashboard::new().await;
    let now = at("2027-03-06T12:00:00Z");
    *dashboard.fixture.runtime.0.lock().unwrap() = now;
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        for item in &mut remote.subscription.items.data {
            item.current_period_start = now.timestamp() - 20_000;
            item.current_period_end = now.timestamp() + 80_000;
        }
        remote.proration = Some(Proration {
            now: now.timestamp(),
            starts_at: now.timestamp() - 20_000,
            ends_at: now.timestamp() + 80_000,
            charges: Vec::new(),
        });
    }
    dashboard.save(&dashboard.paid());
    let payer = dashboard.fixture.login(7, Some(Role::Billing)).await;
    let review = review(&dashboard, &payer).await;
    (dashboard, payer, review)
}

pub(super) async fn review(dashboard: &Dashboard, payer: &Browser) -> Value {
    json(
        dashboard
            .request(
                Some(payer),
                Method::GET,
                "/checkout/review?tier=business&start_blocks=3&machines=0",
                None,
            )
            .await,
        StatusCode::OK,
    )
    .await
}

pub(super) fn payment(review: &Value) -> Value {
    json!({"quote_id":review["quote"]["id"].as_str().unwrap()})
}

pub(super) fn upgrade() -> SubscriptionRequest {
    SubscriptionRequest::Change(
        ChangeMode::Upgrade,
        vec![
            ItemChange::Remove("si_tier"),
            ItemChange::Update("si_starts", "price_starts", 3),
            ItemChange::Remove("si_machine"),
            ItemChange::Add("price_business", 1),
        ],
    )
    .at(1804334400)
}

pub(super) fn business(dashboard: &Dashboard) -> Subscription {
    let mut current = dashboard.fake.state.lock().unwrap().subscription.clone();
    current
        .items
        .data
        .retain(|item| item.price.lookup_key != PriceCode::Machine);
    current.items.data[0].id = "si_business".into();
    current.items.data[0].price.id = "price_business".into();
    current.items.data[0].price.lookup_key = PriceCode::Business;
    current.items.data[1].quantity = 3;
    current
}

pub(super) fn advance(dashboard: &Dashboard, seconds: i64) {
    let now = at("2027-03-06T12:00:00Z") + chrono::Duration::seconds(seconds);
    *dashboard.fixture.runtime.0.lock().unwrap() = now;
    dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .proration
        .as_mut()
        .unwrap()
        .now = now.timestamp();
}
