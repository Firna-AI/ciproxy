//! Runtime wiring protects demand, cost bounds, existing runners, and callback scope.

use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Duration;
use ciproxy_control_plane::boat_callback;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::record::StartPhase;
use tower::ServiceExt;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::support::at;

#[tokio::test]
async fn wait_deadline_starts_boat_once_and_provider_failure_preserves_pending_supply() {
    for (assigned, before, boundary) in [
        (
            Duration::zero(),
            Duration::seconds(29),
            Duration::seconds(30),
        ),
        (
            Duration::milliseconds(500),
            Duration::milliseconds(30_499),
            Duration::milliseconds(30_500),
        ),
    ] {
        let harness = Harness::new(true, true).await;
        harness.queued_at(at() + assigned);
        harness.ports.pending.store(true, Ordering::SeqCst);
        *harness.ports.now.lock().unwrap() = at() + before;
        harness.runtime.tick().await.unwrap();
        assert_eq!(
            harness.ports.creates.lock().unwrap().len(),
            0,
            "assigned offset: {assigned}"
        );
        *harness.ports.now.lock().unwrap() = at() + boundary;
        harness.runtime.tick().await.unwrap();
        assert_eq!(
            harness.ports.creates.lock().unwrap().len(),
            1,
            "assigned offset: {assigned}"
        );
        harness.ports.fail.store(true, Ordering::SeqCst);
        harness.runtime.tick().await.unwrap();
        assert_eq!(
            harness
                .fixture
                .store
                .boat_runner_intents(harness.fixture.workspace)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(harness.ports.creates.lock().unwrap().len(), 1);
        assert_eq!(harness.own_starts.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn no_eligible_machine_starts_boat_immediately_and_retains_its_original_root() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.ports.launches.load(Ordering::SeqCst), 1);
    assert_eq!(
        harness.ports.inputs.lock().unwrap()[0]["configure"]["organization_id"],
        "42"
    );
    let stored = harness
        .fixture
        .store
        .boat_runtime_snapshot(harness.fixture.workspace)
        .unwrap();
    assert_eq!(stored.starts[0].required_root_disk_bytes, 80_899_612_672);
    harness.runtime.image = Some(ImageDownload {
        image_id: ImageId("new-larger-image".to_owned()),
        download_url: Secret::new("https://images.example/new"),
        sha256: "fixture".to_owned(),
        size_bytes: 1,
        root_disk_bytes: 120_000_000_000,
    });
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 1);
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    assert_eq!(
        harness
            .fixture
            .store
            .runners(harness.fixture.workspace, harness.fixture.organization)
            .unwrap()
            .iter()
            .filter(|runner| runner.github_id == Some(100))
            .count(),
        1
    );
}

#[tokio::test]
async fn boat_failure_keeps_free_own_capacity_scheduling() {
    let harness = Harness::new(true, false).await;
    harness.queued();
    harness.ports.fail.store(true, Ordering::SeqCst);
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 0);
}

#[tokio::test]
async fn idle_cleanup_preserves_a_github_busy_race_then_deletes_after_completion() {
    let harness = Harness::new(false, false).await;
    let mut job = harness.queued();
    harness.runtime.tick().await.unwrap();
    job.phase = DemandPhase::Completed;
    job.completed_at = Some(at());
    harness
        .fixture
        .store
        .save_github_job(harness.fixture.workspace, &job, at())
        .unwrap();
    harness.busy.store(true, Ordering::SeqCst);
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    harness.busy.store(false, Ordering::SeqCst);
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(
        harness
            .fixture
            .store
            .boat_runtime_snapshot(harness.fixture.workspace)
            .unwrap()
            .starts[0]
            .phase,
        StartPhase::Ended
    );
}

#[tokio::test]
async fn callback_checks_runner_scope_and_consumes_its_token_before_deletion() {
    let harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.tick().await.unwrap();
    let input = harness.ports.inputs.lock().unwrap()[0].clone();
    let token = input["exit_report_token"].as_str().unwrap();
    let runner = input["configure"]["runner_id"].as_str().unwrap();
    let router = boat_callback::router(harness.service.clone());
    let request = |runner: &str, kind: &str| {
        Request::builder()
            .method("POST")
            .uri("/hooks/boat/runner-exit")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"type":kind,"runner_id":runner,"exit":{"code":0}}).to_string(),
            ))
            .unwrap()
    };
    assert_eq!(
        router
            .clone()
            .oneshot(request(&Uuid::new_v4().to_string(), "runner_exited"))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        router
            .clone()
            .oneshot(request(runner, "runner_ready"))
            .await
            .unwrap()
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        router
            .clone()
            .oneshot(request(runner, "runner_exited"))
            .await
            .unwrap()
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        router
            .oneshot(request(runner, "runner_exited"))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn full_lifetime_pending_bound_blocks_the_next_dispatch_until_limit_is_raised() {
    let harness = Harness::new(false, false).await;
    harness.queued();
    harness.queued();
    let mut settings = harness
        .fixture
        .store
        .settings(harness.fixture.workspace)
        .unwrap();
    settings.boat_spend_limit_usd_micros = Some(600_000);
    harness
        .fixture
        .store
        .save_settings(harness.fixture.workspace, &settings)
        .unwrap();
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 1);
    settings.boat_spend_limit_usd_micros = Some(1_100_000);
    harness
        .fixture
        .store
        .save_settings(harness.fixture.workspace, &settings)
        .unwrap();
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 2);
}
