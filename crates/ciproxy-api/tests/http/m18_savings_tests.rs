//! Real metering report serialization, unknown finish, costs, and UTC ranges.

use std::sync::Arc;

use axum::http::StatusCode;
use ciproxy_api::pages::PageServices;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::metering::history::{BoatCost, MachineCost};
use rust_decimal::Decimal;
use serde_json::Value;
use uuid::Uuid;

use super::fixture_io::json;
use super::m18_jobs_support::{actions, demand, runner};
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};
use super::machine_tests::register;

#[tokio::test]
async fn m18_savings_keeps_exact_minutes_missing_costs_and_signed_display_rounding() {
    let stripe = Stripe::new();
    let mut pages = Pages::new(stripe.provider());
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let (machine, _connection) = register(&pages.fixture, &admin).await;
    pages.registration(machine);
    let own = runner(&pages, Some(machine), 1001);
    let boat = runner(&pages, None, 1002);
    actions(&pages, &own, 1, "2026-11-06T11:59:00Z");
    demand(
        &pages,
        &own,
        None,
        DemandPhase::Completed,
        Some("2026-11-06T11:58:00Z"),
        None,
    );
    let mut job = actions(&pages, &boat, 2, "2026-11-06T11:58:30Z");
    job.completed_at = Some(at("2026-11-06T12:00:00Z"));
    pages
        .fixture
        .database
        .store
        .update_job(pages.fixture.workspace, &job)
        .unwrap();
    pages
        .fixture
        .database
        .store
        .save_boat_cost(
            pages.fixture.workspace,
            &BoatCost {
                runner_id: boat.id,
                starts_at: at("2026-11-06T11:58:30Z"),
                ends_at: Some(at("2026-11-06T12:00:00Z")),
                start_usd: Decimal::new(5, 3),
                minute_usd: Decimal::ZERO,
            },
        )
        .unwrap();
    let response = pages.request(Some(&admin), "GET", "/savings", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let missing = json(response).await;
    assert_eq!(missing["range"], "last30_days");
    assert_eq!(missing["from"], "2026-10-08");
    assert_eq!(missing["until"], "2026-11-07");
    assert_eq!(missing["days"].as_array().unwrap().len(), 30);
    assert_eq!(missing["machines"]["starts"], "2");
    assert_eq!(missing["machines"]["minutes"], "1");
    assert_eq!(missing["boat"]["starts"], "1");
    assert_eq!(missing["boat"]["minutes"], "1.5");
    assert_eq!(missing["total_cents"], Value::Null);
    assert_eq!(missing["comparisons"][0]["cost_cents"], "2");
    assert_eq!(missing["comparisons"][0]["savings_cents"], Value::Null);
    assert_eq!(missing["boat_cents"], "1");
    assert_eq!(missing["known_plan_cents"], "0");
    assert_eq!(missing["can_add_machine_costs"], true);
    pages
        .fixture
        .database
        .store
        .save_machine_cost(
            pages.fixture.workspace,
            &MachineCost {
                id: Uuid::new_v4(),
                machine_id: machine,
                starts_at: at("2026-11-06T00:00:00Z"),
                ends_at: None,
                monthly_usd: Decimal::from(30),
            },
        )
        .unwrap();
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    let full = json(
        pages
            .request(Some(&viewer), "GET", "/savings?range=this_month", None)
            .await,
    )
    .await;
    assert_eq!(full["from"], "2026-11-01");
    assert_eq!(full["known_machine_cents"], "100");
    assert_eq!(full["total_cents"], "101");
    assert_eq!(full["comparisons"][0]["savings_cents"], "-99");
    assert_eq!(full["comparisons"][1]["savings_cents"], "-99");
    assert_eq!(full["lead"], "blacksmith");
    assert_eq!(full["comparisons"][0]["price_list_date"], "2026-11-01");
    assert_eq!(full["can_add_machine_costs"], false);
    assert_eq!(full["days"][5]["machines"]["starts"], "2");
    let previous = json(
        pages
            .request(Some(&viewer), "GET", "/savings?range=last_month", None)
            .await,
    )
    .await;
    assert_eq!(previous["from"], "2026-10-01");
    assert_eq!(previous["until"], "2026-11-01");
    assert_eq!(previous["machines"]["starts"], "0");
    assert_eq!(previous["total_cents"], "0");
    let mut catalogs = pages.services.price_lists.clone().unwrap();
    catalogs[0].sizes[0].minute_usd = "1000000000000000000000000000".parse().unwrap();
    pages.services = Arc::new(PageServices {
        billing: pages.services.billing.clone(),
        metering: pages.services.metering.clone(),
        price_lists: Some(catalogs),
        paid_review: pages.services.paid_review.clone(),
        overflow: pages.services.overflow.clone(),
        boat_root_sizing: pages.services.boat_root_sizing,
    });
    let large = json(
        pages
            .request(Some(&viewer), "GET", "/savings?range=this_month", None)
            .await,
    )
    .await;
    assert_eq!(
        large["comparisons"][0]["cost_cents"],
        "250000000000000000000000000000"
    );
}
