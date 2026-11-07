//! The composed paid review uses real Stripe HTTP previews and the true anchor.

use axum::http::StatusCode;
use ciproxy_billing::catalog::PriceCode;
use ciproxy_billing::stripe::wire::types::{BillingCycleAnchor, Item, Price};
use ciproxy_store_interface::dashboard::records::Role;
use reqwest::Method;
use serde_json::json;
use uuid::Uuid;

use super::fake_subscriptions::{ChangeMode, ItemChange, SubscriptionOutcome, SubscriptionRequest};
use super::preview_support::{DOWNGRADE, UPGRADE, previews};
use super::support::{Dashboard, at, json};

#[tokio::test]
async fn composed_paid_review_uses_stripe_previews_and_recurring_anchor() {
    let dashboard = Dashboard::new().await;
    *dashboard.fixture.runtime.0.lock().unwrap() = at("2027-03-06T12:00:00Z");
    let payer = dashboard.fixture.login(7, Some(Role::Billing)).await;
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        remote.subscription.billing_cycle_anchor = Some(at("2027-01-31T12:00:00Z").timestamp());
        for item in &mut remote.subscription.items.data {
            item.current_period_start = at("2027-02-28T12:00:00Z").timestamp();
            item.current_period_end = at("2027-03-31T12:00:00Z").timestamp();
        }
    }
    dashboard.save(&dashboard.paid());
    previews(&dashboard.fake, true, 31900, UPGRADE);
    let review = json(
        dashboard
            .request(
                Some(&payer),
                Method::GET,
                "/checkout/review?tier=business&start_blocks=3&machines=0",
                None,
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(review["today_cents"], "17431");
    assert_eq!(review["choice"]["monthly_cents"], "31900");
    assert_eq!(review["choice"]["selection"]["add_ons"]["start_blocks"], 3);
    assert_eq!(review["choice"]["selection"]["add_ons"]["machines"], 0);
    assert_eq!(review["billing_day"], 31);
    assert_eq!(review["period_end"], "2027-03-31T12:00:00Z");
    assert_eq!(review["prorated"], true);
    let mut changed = dashboard.fake.state.lock().unwrap().subscription.clone();
    changed
        .items
        .data
        .retain(|item| item.price.lookup_key != PriceCode::Machine);
    changed.items.data[0].id = "si_business".into();
    changed.items.data[0].price = Price {
        id: "price_business".into(),
        lookup_key: PriceCode::Business,
    };
    changed.items.data[1].quantity = 3;
    dashboard.fake.expect_subscription(
        SubscriptionRequest::Change(
            ChangeMode::Upgrade,
            vec![
                ItemChange::Remove("si_tier"),
                ItemChange::Update("si_starts", "price_starts", 3),
                ItemChange::Remove("si_machine"),
                ItemChange::Add("price_business", 1),
            ],
        ),
        SubscriptionOutcome::respond(changed.clone()),
    );
    let response = json(
        dashboard
            .request(
                Some(&payer),
                Method::POST,
                "/billing/change",
                Some(json!({"tier":"business","add_ons":{"start_blocks":3,"machines":0}})),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(response["state"], "confirmed");
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        remote.subscription.billing_cycle_anchor = Some(at("2027-02-28T12:00:00Z").timestamp());
        remote.subscription.billing_cycle_anchor_config =
            Some(BillingCycleAnchor { day_of_month: 31 });
    }
    previews(&dashboard.fake, false, 6900, DOWNGRADE);
    let review = json(
        dashboard
            .request(
                Some(&payer),
                Method::GET,
                "/checkout/review?tier=team&start_blocks=2&machines=0",
                None,
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(review["today_cents"], "0");
    assert_eq!(review["choice"]["monthly_cents"], "6900");
    assert_eq!(review["billing_day"], 31);
    changed.items.data.truncate(1);
    changed.items.data.push(Item {
        id: "si_extra_machine".into(),
        price: Price {
            id: "price_machine".into(),
            lookup_key: PriceCode::Machine,
        },
        quantity: 1,
        current_period_start: at("2027-02-28T12:00:00Z").timestamp(),
        current_period_end: at("2027-03-31T12:00:00Z").timestamp(),
    });
    dashboard.fake.expect_subscription(
        SubscriptionRequest::Change(
            ChangeMode::Prorated,
            vec![
                ItemChange::Update("si_business", "price_business", 1),
                ItemChange::Remove("si_starts"),
                ItemChange::Add("price_machine", 1),
            ],
        ),
        SubscriptionOutcome::respond(changed.clone()),
    );
    let addons = json(
        dashboard
            .request(
                Some(&payer),
                Method::PUT,
                "/billing/add-ons",
                Some(json!({"start_blocks":0,"machines":1})),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(addons["state"], "confirmed");
    let portal = json(
        dashboard
            .request(Some(&payer), Method::POST, "/billing/portal", None)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(portal["url"], "https://billing.stripe.test/portal");
    changed.cancel_at_period_end = true;
    dashboard.fake.expect_subscription(
        SubscriptionRequest::CancelAtPeriodEnd,
        SubscriptionOutcome::respond(changed),
    );
    let free = json(
        dashboard
            .request(Some(&payer), Method::POST, "/billing/move-to-free", None)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(free["ok"], true);
    let billing = json(
        dashboard
            .request(Some(&payer), Method::GET, "/billing", None)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(billing["status"]["cancel_at"], "2027-03-31T12:00:00Z");
    assert_eq!(billing["status"]["selection"]["tier"], "business");
    dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .subscription
        .metadata
        .workspace_id = Some(Uuid::nil());
    assert_eq!(
        dashboard
            .request(
                Some(&payer),
                Method::GET,
                "/checkout/review?tier=team",
                None
            )
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    dashboard.fake.probe().check();
    dashboard.close().await;
}
