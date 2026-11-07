//! Paid review quotes, authorized add-ons, upgrades, and period-end cancellation.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_api::pages::PageServices;
use ciproxy_billing::catalog::{CheckoutReview, PriceCode};
use ciproxy_billing::service::paid_review::{PaidQuote, PaidReviewMock};
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::metering::history::Period;
use serde_json::json;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::fixture_io;
use super::m18_stripe::{Scenario, Stripe};
use super::m18_support::{Pages, at};

#[tokio::test]
async fn m18_paid_review_requires_authoritative_quote_and_uses_its_anchor_day() {
    let stripe = Stripe::new();
    let mut pages = Pages::new(stripe.provider());
    stripe.bind(pages.fixture.workspace);
    *pages.fixture.runtime.0.lock().unwrap() = at("2027-03-06T12:00:00Z");
    let billing = pages.fixture.login(7, Some(Role::Billing)).await;
    let mut state = pages.state(Tier::Team);
    state.paid_period_start = Some(at("2027-01-31T12:00:00Z"));
    state.paid_period_end = Some(at("2027-02-28T12:00:00Z"));
    pages.save(&state);
    assert_eq!(
        pages
            .request(Some(&billing), "GET", "/billing", None)
            .await
            .status(),
        StatusCode::OK
    );
    let absent = pages
        .request(
            Some(&billing),
            "GET",
            "/checkout/review?tier=business",
            None,
        )
        .await;
    assert_eq!(absent.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(fixture_io::json(absent).await["code"], "unavailable");
    let workspace = pages.fixture.workspace;
    pages.services = Arc::new(PageServices {
        boat_root_sizing: None,
        billing: pages.services.billing.clone(),
        metering: pages.services.metering.clone(),
        price_lists: pages.services.price_lists.clone(),
        overflow: None,
        paid_review: Some(Arc::new(Unimock::new(
            PaidReviewMock::review
                .each_call(matching!())
                .answers_arc(Arc::new(move |_, access, selected, time| {
                    assert_eq!(access.workspace_id, workspace);
                    assert_eq!(access.role, ciproxy_billing::access::Role::Billing);
                    assert_eq!(selected.tier, Tier::Business);
                    assert_eq!(time, at("2027-03-06T12:00:00Z"));
                    Ok(PaidQuote {
                        amounts: CheckoutReview {
                            monthly_cents: 28900,
                            today_cents: 21000,
                            billing_day: 31,
                        },
                        period: Period {
                            starts_at: at("2027-02-28T12:00:00Z"),
                            ends_at: at("2027-03-31T12:00:00Z"),
                        },
                    })
                })),
        ))),
    });
    let review = fixture_io::json(
        pages
            .request(
                Some(&billing),
                "GET",
                "/checkout/review?tier=business",
                None,
            )
            .await,
    )
    .await;
    assert_eq!(review["today_cents"], "21000");
    assert_eq!(review["choice"]["monthly_cents"], "28900");
    assert_eq!(review["billing_day"], 31);
    assert_eq!(review["period_end"], "2027-03-31T12:00:00Z");
    assert_eq!(review["prorated"], true);
    assert_eq!(
        pages
            .request(Some(&billing), "GET", "/checkout/review?tier=team", None)
            .await
            .status(),
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn m18_paid_actions_preserve_period_wait_for_upgrade_payment_and_open_portal() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider_for(Scenario::Paid));
    stripe.bind(pages.fixture.workspace);
    let billing = pages.fixture.login(7, Some(Role::Billing)).await;
    pages.save(&pages.state(Tier::Team));
    assert_eq!(
        pages
            .request(Some(&billing), "GET", "/billing", None)
            .await
            .status(),
        StatusCode::OK
    );
    let addons = pages
        .request(
            Some(&billing),
            "PUT",
            "/billing/add-ons",
            Some(json!({"start_blocks":1,"machines":2})),
        )
        .await;
    assert_eq!(addons.status(), StatusCode::OK);
    assert_eq!(fixture_io::json(addons).await["state"], "confirmed");
    let state = pages
        .fixture
        .database
        .store
        .billing(pages.fixture.workspace)
        .unwrap()
        .state;
    assert_eq!(state.add_ons.start_blocks, 1);
    assert_eq!(state.add_ons.machines, 2);
    assert_eq!(state.paid_period_start, Some(at("2026-11-01T00:00:00Z")));
    assert_eq!(state.paid_period_end, Some(at("2026-12-01T00:00:00Z")));
    stripe.payment_required.store(true, Ordering::SeqCst);
    let upgrade = pages
        .request(
            Some(&billing),
            "POST",
            "/billing/change",
            Some(json!({"tier":"business","add_ons":{"start_blocks":0,"machines":0}})),
        )
        .await;
    assert_eq!(upgrade.status(), StatusCode::OK);
    let upgrade = fixture_io::json(upgrade).await;
    assert_eq!(upgrade["state"], "payment_required");
    assert_eq!(upgrade["payment_url"], "https://invoice.stripe.test/pay");
    assert_eq!(
        pages
            .fixture
            .database
            .store
            .billing(pages.fixture.workspace)
            .unwrap()
            .state
            .tier,
        Tier::Team
    );
    let portal = pages
        .request(Some(&billing), "POST", "/billing/portal", None)
        .await;
    assert_eq!(portal.status(), StatusCode::OK);
    assert_eq!(
        fixture_io::json(portal).await["url"],
        "https://billing.stripe.test/m18"
    );
    pages.save(&pages.state(Tier::Team));
    stripe.subscription.lock().unwrap().pending_update = None;
    let free = pages
        .request(Some(&billing), "POST", "/billing/move-to-free", None)
        .await;
    assert_eq!(free.status(), StatusCode::OK);
    let saved = pages
        .fixture
        .database
        .store
        .billing(pages.fixture.workspace)
        .unwrap()
        .state;
    assert_eq!(saved.tier, Tier::Team);
    assert_eq!(saved.cancel_at, Some(at("2026-12-01T00:00:00Z")));
    assert_eq!(saved.paid_period_end, Some(at("2026-12-01T00:00:00Z")));
}

#[tokio::test]
async fn m18_addon_save_preserves_the_reconciled_stripe_tier_after_a_stale_upgrade() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider_for(Scenario::AddOns));
    stripe.bind(pages.fixture.workspace);
    let billing = pages.fixture.login(7, Some(Role::Billing)).await;
    pages.save(&pages.state(Tier::Team));
    {
        let mut saved = pages
            .fixture
            .database
            .store
            .try_lock_billing(pages.fixture.workspace)
            .unwrap()
            .unwrap();
        saved
            .save_command(&BillingCommand {
                id: Uuid::new_v4(),
                started_at: at("2026-11-06T11:59:00Z"),
                payment_expires_at: None,
                kind: CommandKind::Change {
                    selection: Selection {
                        tier: Tier::Business,
                        add_ons: AddOns::default(),
                    },
                    upgrade: true,
                },
            })
            .unwrap();
        let mut current = stripe.subscription.lock().unwrap();
        current.items.data[0].price.id = "price_business".into();
        current.items.data[0].price.lookup_key = PriceCode::Business;
    }
    assert_eq!(
        pages
            .fixture
            .database
            .store
            .billing(pages.fixture.workspace)
            .unwrap()
            .state
            .tier,
        Tier::Team
    );
    let response = pages
        .request(
            Some(&billing),
            "PUT",
            "/billing/add-ons",
            Some(json!({"start_blocks":1,"machines":2})),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(fixture_io::json(response).await["state"], "confirmed");
    let saved = pages
        .fixture
        .database
        .store
        .billing(pages.fixture.workspace)
        .unwrap();
    assert_eq!(saved.state.tier, Tier::Business);
    assert_eq!(
        saved.state.add_ons,
        AddOns {
            start_blocks: 1,
            machines: 2
        }
    );
    assert_eq!(
        saved.state.paid_period_start,
        Some(at("2026-11-01T00:00:00Z"))
    );
    assert_eq!(
        saved.state.paid_period_end,
        Some(at("2026-12-01T00:00:00Z"))
    );
    assert_eq!(
        *stripe.selections.lock().unwrap(),
        vec![Selection {
            tier: Tier::Business,
            add_ons: AddOns {
                start_blocks: 1,
                machines: 2
            }
        }]
    );
}
