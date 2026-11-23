//! Waiting-label notice states for every role through real PostgreSQL and overflow.

use axum::http::StatusCode;
use ciproxy_overflow::error::Error;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::reports::{SlotOccupancy, SlotReport};
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::labels::{LabelPlace, WhenNothingFits};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use rust_decimal::Decimal;
use serde_json::{Value, json};

use super::capacity_fixture;
use super::fixture::Browser;
use super::fixture_io::json;
use super::m18_fit_support::{attach, catalog, sizing};
use super::m18_gap_support;
use super::m18_jobs_support::{demand, runner};
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};
use super::machine_tests::{register, report};

#[tokio::test]
async fn dashboard_reports_the_limit_when_the_next_start_does_not_fit() {
    let mut pages = Pages::new(Stripe::new().provider());
    attach(&mut pages);
    catalog(&pages, true, false);
    m18_gap_support::seed(&pages, Some(Decimal::ZERO));
    pages
        .fixture
        .database
        .store
        .save_overflow_provider_spend_limit(
            pages.fixture.workspace,
            OverflowProvider::Boat,
            Some(6_500_000),
        )
        .unwrap();
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let overflow = runner(&pages, None, 1001);
    demand(&pages, &overflow, None, DemandPhase::Assigned, None, None);

    let body = live(&pages, &admin).await;
    assert_eq!(body["overflow"][0]["counted_cents"], "100");
    assert_eq!(body["overflow"][0]["limit_cents"], "650");
    assert_eq!(body["overflow"][0]["limit_reached"], true);
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["hold"],
        "spend_limit"
    );
    let settings = json(pages.request(Some(&admin), "GET", "/overflow", None).await).await;
    assert_eq!(settings["providers"][0]["spend"]["limit_reached"], true);
}

#[tokio::test]
async fn m18_waiting_label_fit_distinguishes_missing_capacity_busy_refusal_and_fallback() {
    let mut pages = Pages::new(Stripe::new().provider());
    pages.fixture.state.capacity = capacity_fixture::standard();
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let (machine, connection) = register(&pages.fixture, &admin).await;
    let own = runner(&pages, Some(machine), 1001);
    demand(&pages, &own, None, DemandPhase::Assigned, None, None);
    let mut label = pages
        .fixture
        .database
        .store
        .labels(pages.fixture.workspace)
        .unwrap()
        .remove(0);
    label.size.vcpus = 128;
    label.size.memory_gb = 1024;
    pages
        .fixture
        .database
        .store
        .update_label(pages.fixture.workspace, &label)
        .unwrap();
    attach(&mut pages);
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    let billing = pages.fixture.login(9, Some(Role::Billing)).await;
    for (browser, editable, ask) in [
        (&viewer, false, true),
        (&billing, false, true),
        (&admin, true, false),
    ] {
        let body = live(&pages, browser).await;
        let fit = &body["waiting_labels"][0];
        assert_eq!(fit["label_id"], label.id.to_string());
        assert_eq!(fit["waiting_jobs"], "1");
        assert_eq!(fit["nothing_fits"], true);
        assert_eq!(fit["current"]["fit"]["no_machine_fits"], true);
        assert_eq!(
            fit["current"]["fit"]["providers"][0]["fit"]["state"],
            "no_size_fits"
        );
        assert_eq!(fit["current"]["fit"]["providers"][0]["connected"], true);
        assert_eq!(
            fit["current"]["fit"]["fitting_places"],
            serde_json::json!([])
        );
        assert_eq!(fit["can_edit"], editable);
        assert_eq!(fit["ask_admin"], ask);
    }
    let unbounded = m18_gap_support::seed(&pages, None);
    let source = pages
        .services
        .overflow
        .as_ref()
        .unwrap()
        .view(pages.fixture.workspace)
        .await
        .unwrap();
    let source = source.provider(OverflowProvider::Boat).unwrap();
    assert_eq!(source.spend.unbounded_starts, [unbounded]);
    assert!(matches!(
        source.scaler_account(at("2026-11-06T12:00:00Z")),
        Err(Error::AccountingIncomplete)
    ));
    let gap = live(&pages, &viewer).await;
    assert_eq!(gap["overflow"][0]["counted_cents"], Value::Null);
    assert_eq!(gap["overflow"][0]["limit_cents"], "100");
    assert_eq!(gap["overflow"][0]["measured_complete"], false);
    assert_eq!(
        gap["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["state"],
        "no_size_fits"
    );
    assert_eq!(gap["waiting_labels"][0]["nothing_fits"], true);
    assert_eq!(gap["waiting_labels"][0]["ask_admin"], true);
    assert_eq!(
        gap["waiting_labels"][0]["current"]["fit"]["routing"],
        json!({"kind":"waits", "reason":{"kind":"nothing_fits"}})
    );
    catalog(&pages, true, false);
    let body = live(&pages, &viewer).await;
    assert_eq!(body["waiting_labels"][0]["nothing_fits"], false);
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["state"],
        "fitting"
    );
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["fitting_places"],
        serde_json::json!([{"kind":"overflow","provider":"boat"}])
    );
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["size"]["disk_bytes"],
        "100000000000"
    );
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["eligible_size"]["name"],
        "observed-size"
    );
    catalog(&pages, true, true);
    let body = live(&pages, &viewer).await;
    assert_eq!(body["waiting_labels"][0]["nothing_fits"], false);
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["refused"],
        true
    );
    assert_eq!(body["overflow"][0]["refused_sizes"][0], "observed-size");
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["eligible_size"],
        Value::Null
    );
    catalog(&pages, false, false);
    label.when_nothing_fits = WhenNothingFits::LargestOverflow;
    pages
        .fixture
        .database
        .store
        .update_label(pages.fixture.workspace, &label)
        .unwrap();
    let body = live(&pages, &billing).await;
    assert_eq!(body["waiting_labels"][0]["nothing_fits"], false);
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["state"],
        "largest_fallback"
    );
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["size"]["disk_bytes"],
        "60000000000"
    );
    let routing = &body["waiting_labels"][0]["current"]["fit"]["routing"];
    assert_eq!(
        routing,
        &json!({"kind":"waits", "reason":{
            "kind":"provider", "provider":"boat", "hold":"costs_unknown"
        }})
    );
    let settings = json(pages.request(Some(&admin), "GET", "/settings", None).await).await;
    assert_eq!(&settings["labels"][0]["fit"]["routing"], routing);
    pages.fixture.state.capacity = capacity_fixture::unavailable();
    let body = live(&pages, &viewer).await;
    assert_eq!(body["waiting_labels"][0]["nothing_fits"], Value::Null);
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["state"],
        "unavailable"
    );
    pages.fixture.state.capacity = capacity_fixture::standard();
    sizing(&mut pages, false);
    let body = live(&pages, &viewer).await;
    assert_eq!(body["waiting_labels"][0]["nothing_fits"], Value::Null);
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["state"],
        "unavailable"
    );
    label.order.clear();
    pages
        .fixture
        .database
        .store
        .update_label(pages.fixture.workspace, &label)
        .unwrap();
    pages.fixture.state.capacity = capacity_fixture::unavailable();
    let body = live(&pages, &viewer).await;
    assert_eq!(body["waiting_labels"][0]["nothing_fits"], true);
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["providers"][0]["fit"]["state"],
        "disabled"
    );
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["unavailable_machines"],
        1
    );
    label.order = vec![
        LabelPlace::Machine,
        LabelPlace::Provider(OverflowProvider::Boat),
    ];
    pages.fixture.state.capacity = capacity_fixture::standard();
    label.size.vcpus = 2;
    label.size.memory_gb = 8;
    pages
        .fixture
        .database
        .store
        .update_label(pages.fixture.workspace, &label)
        .unwrap();
    let mut observed = report();
    observed.slots = (0..6)
        .map(|index| SlotReport {
            index,
            occupancy: SlotOccupancy::Job {
                runner_id: RunnerId(format!("busy-{index}")),
                job_id: JobId(format!("job-{index}")),
            },
        })
        .collect();
    pages
        .fixture
        .gateway
        .receive(
            connection.session,
            AgentMessage::Report { report: observed },
        )
        .unwrap();
    let body = live(&pages, &viewer).await;
    assert_eq!(body["own_slots_used"], "6");
    assert_eq!(body["own_slots_total"], "6");
    assert_eq!(
        body["waiting_labels"][0]["current"]["fit"]["fitting_machines"],
        1
    );
    assert_eq!(body["waiting_labels"][0]["nothing_fits"], false);
}

async fn live(pages: &Pages, browser: &Browser) -> Value {
    let response = pages
        .request(Some(browser), "GET", "/jobs/live", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json(response).await;
    assert_eq!(body["waiting_labels"].as_array().unwrap().len(), 1);
    body
}
