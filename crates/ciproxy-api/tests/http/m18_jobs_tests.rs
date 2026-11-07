//! HTTP history visibility, trusted aliases, cache metadata, and exact timing.

use axum::http::StatusCode;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::jobs::{CacheSource, JobState, MachineHandledCacheCounts};
use ciproxy_store_interface::scale_sets::ScaleSet;
use serde_json::Value;
use uuid::Uuid;

use super::fixture_io::json;
use super::m18_job_requests_support::own_runner;
use super::m18_jobs_support::{actions, demand, demand_at, runner};
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, capacity};
use super::machine_tests::register;

#[tokio::test]
async fn m18_history_applies_plan_window_aliases_and_actual_job_metadata() {
    let stripe = Stripe::new();
    let mut pages = Pages::new(stripe.provider());
    stripe.bind(pages.fixture.workspace);
    pages.fixture.state.capacity = capacity();
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let (machine, _connection) = register(&pages.fixture, &admin).await;
    let own = own_runner(&pages, machine, 1001);
    let boat = runner(&pages, None, 1002);
    let mut recent = actions(&pages, &own, 11, "2026-11-06T11:59:00Z");
    recent.state = JobState::Running;
    recent.result = None;
    recent.completed_at = None;
    recent.cache_source = Some(CacheSource::MachineAndGithub);
    recent.machine_handled_cache_counts = Some(MachineHandledCacheCounts {
        restored_entries: 5,
        restored_bytes: 9_007_199_254_740_993,
    });
    pages
        .fixture
        .database
        .store
        .update_job(pages.fixture.workspace, &recent)
        .unwrap();
    let mut completion = demand(
        &pages,
        &own,
        Some(11),
        DemandPhase::Completed,
        Some("2026-11-06T11:59:00Z"),
        Some("2026-11-06T12:00:00Z"),
    );
    completion.result = Some("failed".into());
    pages
        .fixture
        .database
        .store
        .save_github_job(
            pages.fixture.workspace,
            &completion,
            super::m18_support::at("2026-11-06T12:00:00Z"),
        )
        .unwrap();
    let mut late = actions(&pages, &own, 16, "2026-10-29T12:00:00Z");
    late.result = None;
    late.completed_at = None;
    pages
        .fixture
        .database
        .store
        .update_job(pages.fixture.workspace, &late)
        .unwrap();
    demand_at(
        &pages,
        &own,
        Some(16),
        DemandPhase::Completed,
        Some("2026-10-29T12:00:00Z"),
        None,
        "2026-10-29T11:59:50Z",
    );
    actions(&pages, &own, 12, "2026-10-29T12:00:00Z");
    actions(&pages, &own, 13, "2026-10-02T12:00:00Z");
    actions(&pages, &own, 14, "2026-08-07T12:00:00Z");
    demand(
        &pages,
        &boat,
        None,
        DemandPhase::Completed,
        Some("2026-11-06T11:59:00Z"),
        None,
    );
    let mut sparse = actions(&pages, &own, 15, "2026-11-06T11:59:30Z");
    sparse.state = JobState::Queued;
    sparse.started_at = None;
    sparse.completed_at = None;
    pages
        .fixture
        .database
        .store
        .update_job(pages.fixture.workspace, &sparse)
        .unwrap();
    demand(
        &pages,
        &own,
        Some(15),
        DemandPhase::Started,
        Some("2026-11-06T11:59:30Z"),
        None,
    );
    let mut organization = pages.fixture.organization.clone();
    organization.id = Uuid::new_v4();
    organization.github_id = 200;
    organization.installation_id = 201;
    organization.login = "other-org".into();
    pages
        .fixture
        .database
        .store
        .install_organization(&organization)
        .unwrap();
    let scale = ScaleSet {
        id: Uuid::new_v4(),
        organization_id: organization.id,
        label_id: own.label_id,
        github_id: 502,
        runner_group_id: 47,
    };
    pages
        .fixture
        .database
        .store
        .create_scale_set(pages.fixture.workspace, &scale)
        .unwrap();
    let mut other = own.clone();
    other.id = Uuid::new_v4();
    other.organization_id = organization.id;
    other.scale_set_id = scale.id;
    other.github_id = Some(1005);
    pages
        .fixture
        .database
        .store
        .create_runner(pages.fixture.workspace, &other)
        .unwrap();
    let second = actions(&pages, &other, 21, "2026-11-06T11:59:00Z");
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    let response = pages
        .request(Some(&viewer), "GET", "/jobs/history", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let history = json(response).await;
    assert_eq!(history["history_days"], 7);
    assert_eq!(history["jobs"].as_array().unwrap().len(), 4);
    assert_eq!(find(&history, "job-21")["organization"], "other-org");
    let second_detail = json(
        pages
            .request(Some(&viewer), "GET", &format!("/jobs/{}", second.id), None)
            .await,
    )
    .await;
    assert_eq!(
        second_detail["job"]["organization_id"],
        organization.id.to_string()
    );
    let live = json(
        pages
            .request(Some(&viewer), "GET", "/jobs/live", None)
            .await,
    )
    .await;
    assert_eq!(live["running"].as_array().unwrap().len(), 1);
    assert_eq!(live["running"][0]["counts_as_start"], true);
    assert_eq!(live["waiting_count"], "0");
    let own_row = find(&history, "job-11");
    assert_eq!(own_row["result"], "failed");
    assert_eq!(own_row["status"], "completed");
    assert_eq!(own_row["completed_at"], "2026-11-06T12:00:00Z");
    assert_eq!(own_row["actual_size"]["disk_bytes"], "58300000001");
    assert_eq!(own_row["label"]["disk_gb"], 20);
    assert_eq!(own_row["cache_source"], "machine_and_github");
    assert_eq!(
        own_row["cache_counts"]["restored_bytes"],
        "9007199254740993"
    );
    assert_eq!(own_row["waited_seconds"], "10");
    assert_eq!(own_row["runtime_seconds"], "60");
    assert_eq!(own_row["counts_as_start"], true);
    assert_eq!(own_row["head_branch"], "main");
    assert_eq!(
        own_row["github_url"],
        "https://github.com/customer/private/actions/runs/901/job/11"
    );
    let observed = find(&history, "job-16");
    assert_eq!(observed["status"], "completed");
    assert_eq!(observed["completed_at"], Value::Null);
    assert_eq!(observed["result"], "succeeded");
    assert_eq!(find(&history, "job-21")["result"], "success");
    let unknown = find(&history, "scale-job");
    assert_eq!(unknown["status"], "completed");
    assert_eq!(unknown["completed_at"], Value::Null);
    assert_eq!(unknown["runtime_seconds"], Value::Null);
    assert_eq!(unknown["counts_as_start"], true);
    assert_eq!(unknown["location"]["size"], "actual-medium");
    assert_eq!(unknown["cache_counts"], Value::Null);
    let detail = json(
        pages
            .request(Some(&viewer), "GET", &format!("/jobs/{}", recent.id), None)
            .await,
    )
    .await;
    assert_eq!(detail["job"]["id"], recent.id.to_string());
    assert_eq!(detail["job"]["result"], "failed");
    assert_eq!(detail["job"]["status"], "completed");
    assert_eq!(detail["job"]["completed_at"], "2026-11-06T12:00:00Z");
    for (tier, count, days) in [(Tier::Team, 5, 30), (Tier::Business, 6, 90)] {
        pages.save(&pages.state(tier));
        let response = json(
            pages
                .request(Some(&viewer), "GET", "/jobs/history", None)
                .await,
        )
        .await;
        assert_eq!(response["jobs"].as_array().unwrap().len(), count);
        assert_eq!(response["history_days"], days);
    }
    pages.save(&pages.state(Tier::Free));
    let old = pages
        .fixture
        .database
        .store
        .jobs(pages.fixture.workspace, pages.fixture.organization.id)
        .unwrap()
        .into_iter()
        .find(|job| job.github_id == 12)
        .unwrap();
    assert_eq!(
        pages
            .request(Some(&viewer), "GET", &format!("/jobs/{}", old.id), None)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    let filtered = json(
        pages
            .request(
                Some(&viewer),
                "GET",
                "/jobs/history?place=boat&search=SCALE&from=2026-11-06&until=2026-11-07",
                None,
            )
            .await,
    )
    .await;
    assert_eq!(filtered["jobs"].as_array().unwrap().len(), 1);
    assert_eq!(
        pages
            .fixture
            .database
            .store
            .jobs(pages.fixture.workspace, pages.fixture.organization.id)
            .unwrap()
            .len(),
        6
    );
}

fn find<'a>(history: &'a Value, name: &str) -> &'a Value {
    history["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["name"] == name)
        .unwrap()
}
