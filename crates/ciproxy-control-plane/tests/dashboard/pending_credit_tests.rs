//! An earlier machine credit must reach the reviewed paid move exactly once.

use axum::http::StatusCode;
use ciproxy_billing::catalog::PriceCode;
use ciproxy_store_interface::billing::payments::{InvoiceAudit, Settlement};
use ciproxy_store_interface::dashboard::records::Role;
use reqwest::Method;
use serde_json::json;

use super::credit_test_support::work;
use super::fake_credits::CreditLedger;
use super::fake_requests::Proration;
use super::fake_subscriptions::{ChangeMode, ItemChange, SubscriptionOutcome, SubscriptionRequest};
use super::fixture::Browser;
use super::support::{Dashboard, at, json};

#[tokio::test]
async fn paid_move_charges_the_reviewed_amount_with_an_earlier_machine_credit() {
    let (dashboard, payer) = machine_credit().await;
    let review = json(
        dashboard
            .request(
                Some(&payer),
                Method::GET,
                "/checkout/review?tier=business&start_blocks=0&machines=0",
                None,
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(review["today_cents"], "11500");
    let mut business = dashboard.fake.state.lock().unwrap().subscription.clone();
    business.items.data[0].id = "si_business".into();
    business.items.data[0].price.lookup_key = PriceCode::Business;
    business.items.data[0].price.id = "price_business".into();
    dashboard.fake.expect_subscription(
        SubscriptionRequest::Change(
            ChangeMode::Upgrade,
            vec![
                ItemChange::Remove("si_tier"),
                ItemChange::Add("price_business", 1),
            ],
        )
        .at(1795089600)
        .offset(-1000),
        SubscriptionOutcome::respond(business),
    );
    let response = json(
        dashboard
            .request(
                Some(&payer),
                Method::POST,
                "/billing/change",
                Some(json!({"quote_id":review["quote"]["id"]})),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(response["state"], "confirmed");
    let charges = dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .credits
        .as_ref()
        .unwrap()
        .charges
        .clone();
    assert_eq!(charges, vec![11500]);
    let payment = work(&dashboard, &review);
    assert_eq!(payment.settlement, Settlement::Consumed);
    assert_eq!(
        payment.audit,
        InvoiceAudit::Matched {
            amount_cents: 11500
        }
    );
    assert!(!payment.pending);
    let markers = dashboard
        .fake
        .posts("/v1/subscriptions/sub_recorded")
        .into_iter()
        .filter(|request| request.payment_marker())
        .collect::<Vec<_>>();
    assert_eq!(markers.len(), 1);
    assert_eq!(
        markers[0].value("metadata[ciproxy_quote_id]"),
        review["quote"]["id"].as_str()
    );
    let id = payment.id.to_string();
    assert_eq!(
        markers[0].value("metadata[ciproxy_payment_id]"),
        Some(id.as_str())
    );
    dashboard.close().await;
}

pub(super) async fn machine_credit() -> (Dashboard, Browser) {
    let dashboard = Dashboard::new().await;
    *dashboard.fixture.runtime.0.lock().unwrap() = at("2026-11-19T12:00:00Z");
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        remote.subscription.items.data.remove(1);
        remote.subscription.billing_cycle_anchor = Some(1793793600);
        remote.proration = Some(Proration {
            now: 1795089600,
            starts_at: 1793793600,
            ends_at: 1796385600,
            charges: Vec::new(),
        });
        remote.credits = Some(CreditLedger::new());
    }
    let mut paid = dashboard.paid();
    paid.add_ons.start_blocks = 0;
    dashboard.save(&paid);
    let payer = dashboard.fixture.login(7, Some(Role::Billing)).await;
    let mut team = dashboard.fake.state.lock().unwrap().subscription.clone();
    team.items.data.truncate(1);
    dashboard.fake.expect_subscription(
        SubscriptionRequest::Change(
            ChangeMode::Prorated,
            vec![
                ItemChange::Update("si_tier", "price_team", 1),
                ItemChange::Remove("si_machine"),
            ],
        ),
        SubscriptionOutcome::respond(team),
    );
    let response = json(
        dashboard
            .request(
                Some(&payer),
                Method::PUT,
                "/billing/add-ons",
                Some(json!({"start_blocks":0,"machines":0})),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(response["state"], "confirmed");
    (dashboard, payer)
}
