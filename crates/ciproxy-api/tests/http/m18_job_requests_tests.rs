//! Job request history survives label edits and preserves missing execution facts.

use axum::http::StatusCode;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use serde_json::{Value, json};

use super::capacity_fixture;
use super::fixture_io::json;
use super::m18_job_requests_support::{boat_runner, own_runner};
use super::m18_jobs_support::{actions, demand, runner};
use super::m18_stripe::Stripe;
use super::m18_support::Pages;
use super::machine_tests::register;

#[tokio::test]
async fn m18_jobs_keep_execution_requests_after_label_edit() {
    let mut pages = Pages::new(Stripe::new().provider());
    pages.fixture.state.capacity = capacity_fixture::standard();
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let (machine, _connection) = register(&pages.fixture, &admin).await;
    let own = own_runner(&pages, machine, 1001);
    let boat = boat_runner(&pages, 1002);
    let completed = actions(&pages, &own, 11, "2026-11-06T11:59:00Z");
    let boat_job = demand(
        &pages,
        &boat,
        None,
        DemandPhase::Completed,
        Some("2026-11-06T11:59:00Z"),
        Some("2026-11-06T12:00:00Z"),
    );
    let running = own_runner(&pages, machine, 1003);
    let running_job = demand(
        &pages,
        &running,
        None,
        DemandPhase::Started,
        Some("2026-11-06T11:59:00Z"),
        None,
    );
    let queued = demand(&pages, &own, None, DemandPhase::Assigned, None, None);
    let unknown_own = runner(&pages, Some(machine), 1004);
    let unknown_boat = runner(&pages, None, 1005);
    let unknown_own_job = actions(&pages, &unknown_own, 12, "2026-11-06T11:59:00Z");
    let unknown_boat_job = actions(&pages, &unknown_boat, 13, "2026-11-06T11:59:00Z");
    let mut unplaced = actions(&pages, &own, 14, "2026-11-06T11:59:00Z");
    unplaced.runner_id = None;
    pages
        .fixture
        .database
        .store
        .update_job(pages.fixture.workspace, &unplaced)
        .unwrap();
    let before_response = pages
        .request(Some(&admin), "GET", "/jobs/history", None)
        .await;
    assert_eq!(before_response.status(), StatusCode::OK);
    let before = json(before_response).await;
    let boat_job_id = before["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["name"] == boat_job.name)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let before_live_response = pages.request(Some(&admin), "GET", "/jobs/live", None).await;
    assert_eq!(before_live_response.status(), StatusCode::OK);
    let before_live = json(before_live_response).await;
    assert_eq!(before_live["running"][0]["name"], running_job.name);
    assert_eq!(before_live["waiting"][0]["name"], queued.name);
    let edited = pages
        .request(
            Some(&admin),
            "PUT",
            &format!("/labels/{}", own.label_id),
            Some(json!({
                "name":"ciproxy", "vcpus":8, "memory_gb":32, "disk_gb":80,
                "allow_machines":true, "allow_boat":true, "when_nothing_fits":"wait"
            })),
        )
        .await;
    assert_eq!(edited.status(), StatusCode::OK);
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    let history_response = pages
        .request(Some(&viewer), "GET", "/jobs/history", None)
        .await;
    assert_eq!(history_response.status(), StatusCode::OK);
    let history = json(history_response).await;
    assert_eq!(history["jobs"].as_array().unwrap().len(), 5);
    for id in [completed.id.to_string(), boat_job_id.clone()] {
        let row = history["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|job| job["id"] == id)
            .unwrap();
        assert_eq!(row["label"]["vcpus"], 2, "history request for {id}");
        assert_eq!(row["label"]["memory_gb"], 8);
        assert_eq!(row["label"]["disk_gb"], 20);
        let detail_response = pages
            .request(Some(&viewer), "GET", &format!("/jobs/{id}"), None)
            .await;
        assert_eq!(detail_response.status(), StatusCode::OK);
        let detail = json(detail_response).await;
        assert_eq!(
            detail["job"]["label"]["vcpus"], 2,
            "detail request for {id}"
        );
        assert_eq!(detail["job"]["label"]["memory_gb"], 8);
        assert_eq!(detail["job"]["label"]["disk_gb"], 20);
    }
    let boat_row = history["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == boat_job_id)
        .unwrap();
    assert_eq!(boat_row["actual_size"]["vcpus"], 4);
    assert_eq!(boat_row["actual_size"]["memory_bytes"], "16000000000");
    assert_eq!(boat_row["actual_size"]["disk_bytes"], "100000000000");
    for id in [unknown_own_job.id, unknown_boat_job.id, unplaced.id] {
        let row = history["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|job| job["id"] == id.to_string())
            .unwrap();
        assert_eq!(
            row["label"],
            json!({"name":"ciproxy", "vcpus":null, "memory_gb":null, "disk_gb":null}),
            "unknown history request for {id}",
        );
        let detail_response = pages
            .request(Some(&viewer), "GET", &format!("/jobs/{id}"), None)
            .await;
        assert_eq!(detail_response.status(), StatusCode::OK);
        assert_eq!(json(detail_response).await["job"]["label"], row["label"]);
    }
    let live_response = pages
        .request(Some(&viewer), "GET", "/jobs/live", None)
        .await;
    assert_eq!(live_response.status(), StatusCode::OK);
    let live = json(live_response).await;
    assert_eq!(live["running"].as_array().unwrap().len(), 1);
    assert_eq!(live["running"][0]["id"], before_live["running"][0]["id"]);
    assert_eq!(
        live["running"][0]["label"],
        json!({"name":"ciproxy", "vcpus":2, "memory_gb":8, "disk_gb":20}),
    );
    assert_eq!(live["waiting"].as_array().unwrap().len(), 1);
    assert_eq!(live["waiting"][0]["id"], before_live["waiting"][0]["id"]);
    assert_eq!(
        live["waiting"][0]["label"],
        json!({"name":"ciproxy", "vcpus":8, "memory_gb":32, "disk_gb":80}),
    );
    assert_eq!(live["waiting"][0]["actual_size"], Value::Null);
}
