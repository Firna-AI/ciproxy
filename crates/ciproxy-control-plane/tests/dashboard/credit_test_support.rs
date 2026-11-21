//! Shared public HTTP operations for pending-credit recovery regressions.

use axum::http::StatusCode;
use ciproxy_billing::catalog::PriceCode;
use ciproxy_billing::stripe::wire::types::Subscription;
use ciproxy_store_interface::billing::payments::PaymentWork;
use reqwest::{Method, Response};
use serde_json::{Value, json};

use super::fake_subscriptions::{ChangeMode, ItemChange, SubscriptionRequest};
use super::fixture::Browser;
use super::support::{Dashboard, at, json};

pub(super) async fn review(dashboard: &Dashboard, payer: &Browser) -> Value {
    json(
        dashboard
            .request(
                Some(payer),
                Method::GET,
                "/checkout/review?tier=business&start_blocks=0&machines=0",
                None,
            )
            .await,
        StatusCode::OK,
    )
    .await
}

pub(super) async fn pay(dashboard: &Dashboard, payer: &Browser, quote: &Value) -> Response {
    dashboard
        .request(
            Some(payer),
            Method::POST,
            "/billing/change",
            Some(json!({"quote_id":quote["quote"]["id"]})),
        )
        .await
}

pub(super) fn upgrade(timestamp: i64, offset: i64) -> SubscriptionRequest {
    SubscriptionRequest::Change(
        ChangeMode::Upgrade,
        vec![
            ItemChange::Remove("si_tier"),
            ItemChange::Add("price_business", 1),
        ],
    )
    .at(timestamp)
    .offset(offset)
}

pub(super) fn business(dashboard: &Dashboard) -> Subscription {
    let mut current = dashboard.fake.state.lock().unwrap().subscription.clone();
    current.items.data[0].id = "si_business".into();
    current.items.data[0].price.lookup_key = PriceCode::Business;
    current.items.data[0].price.id = "price_business".into();
    current
}

pub(super) fn work(dashboard: &Dashboard, quote: &Value) -> PaymentWork {
    let mut session = dashboard
        .fixture
        .database
        .store
        .try_lock_billing(dashboard.fixture.workspace)
        .unwrap()
        .unwrap();
    let quote = session
        .quote(quote["quote"]["id"].as_str().unwrap().parse().unwrap())
        .unwrap()
        .unwrap();
    session.payment(quote.command_id.unwrap()).unwrap().unwrap()
}

pub(super) fn set_time(dashboard: &Dashboard, value: &str) {
    *dashboard.fixture.runtime.0.lock().unwrap() = at(value);
    dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .proration
        .as_mut()
        .unwrap()
        .now = at(value).timestamp();
}

pub(super) fn void_invoice(dashboard: &Dashboard) {
    let mut remote = dashboard.fake.state.lock().unwrap();
    let invoice = {
        let invoice = remote
            .credits
            .as_mut()
            .unwrap()
            .invoices
            .last_mut()
            .unwrap();
        invoice["status"] = json!("void");
        invoice["amount_paid"] = json!(0);
        invoice["amount_remaining"] = json!(0);
        invoice.clone()
    };
    remote.subscription.pending_update = None;
    remote.subscription.latest_invoice = Some(serde_json::from_value(invoice.clone()).unwrap());
    remote.raw_reads.insert(
        format!("/v1/invoices/{}", invoice["id"].as_str().unwrap()),
        invoice,
    );
}

pub(super) fn priced_requests(dashboard: &Dashboard) -> usize {
    dashboard
        .fake
        .subscription_mutations()
        .iter()
        .filter(|request| request.value("payment_behavior") == Some("pending_if_incomplete"))
        .count()
}
