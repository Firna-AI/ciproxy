//! Live queue figures, assignment attempts, App visibility, and unavailable sources.

use axum::http::StatusCode;
use ciproxy_api::runtime::Runtime;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::runners::RunnerState;
use serde_json::Value;

use super::capacity_fixture;
use super::fixture_io::json;
use super::m18_jobs_support::{demand, runner};
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};
use super::machine_tests::{register, report};

#[tokio::test]
async fn m18_live_counts_real_slots_attempts_and_marks_unknown_waiting_visibility() {
    let stripe = Stripe::new();
    let mut pages = Pages::new(stripe.provider());
    pages.fixture.state.capacity = capacity_fixture::standard();
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let (machine, connection) = register(&pages.fixture, &admin).await;
    let own = runner(&pages, Some(machine), 1001);
    let boat = runner(&pages, None, 1002);
    demand(
        &pages,
        &boat,
        None,
        DemandPhase::Started,
        Some("2026-11-06T11:59:00Z"),
        None,
    );
    let mut attempt = demand(
        &pages,
        &own,
        None,
        DemandPhase::Completed,
        Some("2026-11-06T11:58:30Z"),
        Some("2026-11-06T11:59:00Z"),
    );
    attempt.assigned_at = Some(at("2026-11-06T11:59:01Z"));
    attempt.phase = DemandPhase::Started;
    attempt.started_at = Some(at("2026-11-06T11:59:30Z"));
    attempt.completed_at = None;
    pages
        .fixture
        .database
        .store
        .save_github_job(
            pages.fixture.workspace,
            &attempt,
            pages.fixture.runtime.now(),
        )
        .unwrap();
    let warm = runner(&pages, Some(machine), 1003);
    pages
        .fixture
        .database
        .store
        .update_runner(
            pages.fixture.workspace,
            warm.organization_id,
            warm.id,
            RunnerState::Ready,
            warm.github_id,
        )
        .unwrap();
    let starting = runner(&pages, None, 1004);
    pages
        .fixture
        .database
        .store
        .update_runner(
            pages.fixture.workspace,
            starting.organization_id,
            starting.id,
            RunnerState::Starting,
            starting.github_id,
        )
        .unwrap();
    demand(&pages, &starting, None, DemandPhase::Assigned, None, None);
    let mut observed = report();
    observed.slots[0].occupancy = SlotOccupancy::Job {
        runner_id: RunnerId(own.id.to_string()),
        job_id: JobId(attempt.id.to_string()),
    };
    pages
        .fixture
        .gateway
        .receive(
            connection.session,
            AgentMessage::Report { report: observed },
        )
        .unwrap();
    let response = pages.request(Some(&admin), "GET", "/jobs/live", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let live = json(response).await;
    assert_eq!(live["running"].as_array().unwrap().len(), 2);
    assert_eq!(live["waiting_count"], "1");
    assert_eq!(live["own_slots_used"], "1");
    assert_eq!(live["own_slots_total"], "6");
    assert_eq!(live["on_boat"], "1");
    assert_eq!(live["warm_runners"], "1");
    assert_eq!(live["starting_runners"].as_array().unwrap().len(), 1);
    assert_eq!(live["overflow"], Value::Null);
    assert_eq!(live["waiting"][0]["counts_as_start"], false);
    assert_eq!(live["waiting"][0]["location"], Value::Null);
    let history = json(
        pages
            .request(Some(&admin), "GET", "/jobs/history", None)
            .await,
    )
    .await;
    assert_eq!(history["jobs"].as_array().unwrap().len(), 1);
    pages
        .fixture
        .database
        .store
        .set_app_suspended(pages.fixture.workspace, pages.fixture.organization.id, true)
        .unwrap();
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    let suspended = json(
        pages
            .request(Some(&viewer), "GET", "/jobs/live", None)
            .await,
    )
    .await;
    assert_eq!(suspended["waiting_count"], Value::Null);
    assert_eq!(suspended["visibility"][0]["suspended"], true);
    assert_eq!(suspended["can_manage"], false);
}
