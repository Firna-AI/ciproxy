//! Canonical slot links and trusted release facts through the real PgStore router.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_api::agent_releases::AgentReleaseClient;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::reports::{SlotOccupancy, SlotReport};
use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::jobs::{JobResult, JobState};
use reqwest::{Client, redirect};
use serde_json::Value;
use uuid::Uuid;

use super::capacity_fixture;
use super::fixture_io::json;
use super::m18_jobs_support::{actions, demand, runner};
use super::m18_stripe::Stripe;
use super::m18_support::Pages;
use super::m20_release_peer::Peer;
use super::machine_tests::{register, report};

#[tokio::test]
async fn m20_slots_resolve_exact_aliases_and_release_failures_stay_explicit() {
    let peer = Peer::new().await;
    let mut pages = Pages::new(Stripe::new().provider());
    pages.fixture.releases = Arc::new(AgentReleaseClient::with_client(
        peer.origin.clone(),
        Client::builder()
            .redirect(redirect::Policy::none())
            .build()
            .unwrap(),
    ));
    pages.fixture.state.capacity = capacity_fixture::standard();
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let (machine, connection) = register(&pages.fixture, &admin).await;
    let own = runner(&pages, Some(machine), 1001);
    pages
        .fixture
        .database
        .store
        .update_runner_size(
            pages.fixture.workspace,
            own.organization_id,
            own.id,
            VmSize {
                vcpu: 4,
                memory_bytes: 16_000_000_000,
                disk_bytes: 80_000_000_000,
            },
        )
        .unwrap();
    let mut action = actions(&pages, &own, 501, "2026-11-06T11:58:00Z");
    action.state = JobState::Running;
    action.completed_at = None;
    action.result = None;
    pages
        .fixture
        .database
        .store
        .update_job(pages.fixture.workspace, &action)
        .unwrap();
    let mut source = demand(
        &pages,
        &own,
        Some(501),
        DemandPhase::Started,
        Some("2026-11-06T11:58:00Z"),
        None,
    );
    let mut observed = report();
    observed.slots = vec![
        occupied(2, own.id, source.id),
        occupied(3, own.id, source.id),
        occupied(4, own.id, source.id),
        occupied(5, Uuid::new_v4(), source.id),
    ];
    pages
        .fixture
        .gateway
        .receive(
            connection.session,
            AgentMessage::Report {
                report: observed.clone(),
            },
        )
        .unwrap();
    let path = pages.fixture.path(&format!("/machines/{machine}"));
    for role in [Role::Viewer, Role::Billing, Role::Admin] {
        let browser = pages
            .fixture
            .login(
                10 + i64::from(role == Role::Admin) + i64::from(role == Role::Billing) * 2,
                Some(role),
            )
            .await;
        let view = json(
            pages
                .fixture
                .request(Some(&browser), "GET", &path, None)
                .await,
        )
        .await;
        assert_eq!(view["latest_agent_version"], "0.2.0");
        assert_eq!(view["agent_up_to_date"], false);
        assert_eq!(view["occupancy"][0]["job"]["id"], action.id.to_string());
        assert_eq!(view["occupancy"][0]["job"]["name"], "job-501");
        assert_eq!(
            view["occupancy"][0]["job"]["repository"],
            "customer/private"
        );
        assert_eq!(
            view["occupancy"][0]["job"]["github_url"],
            "https://github.com/customer/private/actions/runs/901/job/501"
        );
        assert_eq!(view["occupancy"][0]["job_id"], source.id.to_string());
        assert_eq!(view["occupancy"][1]["job"]["id"], action.id.to_string());
        assert_eq!(view["occupancy"][2]["job"], Value::Null);
        assert_eq!(view["occupancy"][3]["job"], Value::Null);
    }
    action.state = JobState::Completed;
    action.completed_at = Some("2026-11-06T11:59:00Z".parse().unwrap());
    action.result = Some(JobResult::Success);
    pages
        .fixture
        .database
        .store
        .update_job(pages.fixture.workspace, &action)
        .unwrap();
    source.phase = DemandPhase::Completed;
    source.completed_at = action.completed_at;
    source.result = Some("succeeded".into());
    pages
        .fixture
        .database
        .store
        .save_github_job(
            pages.fixture.workspace,
            &source,
            "2026-11-06T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
    let finished = json(
        pages
            .fixture
            .request(Some(&admin), "GET", &path, None)
            .await,
    )
    .await;
    assert_eq!(finished["occupancy"][0]["job"]["id"], action.id.to_string());
    assert_eq!(finished["occupancy"][0]["job"]["status"], "completed");
    let (other_machine, other_connection) = register(&pages.fixture, &admin).await;
    pages
        .fixture
        .gateway
        .receive(
            other_connection.session,
            AgentMessage::Report {
                report: observed.clone(),
            },
        )
        .unwrap();
    let other = json(
        pages
            .fixture
            .request(
                Some(&admin),
                "GET",
                &pages.fixture.path(&format!("/machines/{other_machine}")),
                None,
            )
            .await,
    )
    .await;
    assert_eq!(other["occupancy"][0]["job"], Value::Null);
    let before = peer.hits.load(Ordering::SeqCst);
    assert_eq!(
        pages
            .fixture
            .request(
                Some(&admin),
                "GET",
                &format!("/workspaces/{}/machines/{machine}", Uuid::new_v4()),
                None
            )
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(peer.hits.load(Ordering::SeqCst), before);
    peer.status.store(503, Ordering::SeqCst);
    pages.fixture.advance(Duration::seconds(60));
    let offline = json(
        pages
            .fixture
            .request(Some(&admin), "GET", &path, None)
            .await,
    )
    .await;
    assert_eq!(offline["health_old"], true);
    assert_eq!(offline["agent_version"], "0.1.0");
    assert_eq!(offline["latest_agent_version"], Value::Null);
    assert_eq!(offline["agent_up_to_date"], Value::Null);
    assert_eq!(offline["cache"]["docker_repositories"], "2");
    pages.fixture.state.capacity = capacity_fixture::unavailable();
    let unknown = json(
        pages
            .fixture
            .request(Some(&admin), "GET", &path, None)
            .await,
    )
    .await;
    assert_eq!(unknown["disk_limited"], Value::Null);
    assert_eq!(unknown["default_jobs_by_slots"], "6");
    assert_eq!(unknown["default_root_disk_bytes"], Value::Null);
    peer.status.store(200, Ordering::SeqCst);
    observed.agent_version = "0.2.0".into();
    pages
        .fixture
        .gateway
        .receive(
            connection.session,
            AgentMessage::Report { report: observed },
        )
        .unwrap();
    let updated = json(
        pages
            .fixture
            .request(Some(&admin), "GET", &path, None)
            .await,
    )
    .await;
    assert_eq!(updated["agent_up_to_date"], true);
    assert_eq!(updated["health_old"], false);
    pages.fixture.advance(Duration::days(8));
    let expired = json(
        pages
            .fixture
            .request(Some(&admin), "GET", &path, None)
            .await,
    )
    .await;
    assert_eq!(expired["occupancy"][0]["job"], Value::Null);
}

fn occupied(index: u64, runner: Uuid, job: Uuid) -> SlotReport {
    SlotReport {
        index,
        occupancy: SlotOccupancy::Job {
            runner_id: RunnerId(runner.to_string()),
            job_id: JobId(job.to_string()),
        },
    }
}
