//! Provider placement uses the configured factory and image from the registry.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_control_plane::overflow_callback;
use ciproxy_store_interface::labels::LabelPlace;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use super::overflow_service::{attach, post_exit_at};
use super::overflow_support::Harness;
use super::settled::Settled;
use super::support::at;

#[tokio::test]
async fn provider_first_label_uses_the_configured_executor_and_image() {
    let mut harness = Harness::new(true, false).await;
    let root_sizing = RootSizing {
        image_root_bytes: 47_000_000_000,
    };
    harness.service = attach(
        &mut harness.runtime,
        &harness.fixture,
        &harness.ports,
        root_sizing,
    );
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;
    let mut label = store.labels(workspace).unwrap().remove(0);
    label.order = vec![
        LabelPlace::Provider(OverflowProvider::Boat),
        LabelPlace::Machine,
    ];
    store.update_label(workspace, &label).unwrap();
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();

    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 0);
    let intent = store.overflow_runner_intents(workspace).unwrap().remove(0);
    let start = store
        .overflow_runtime_snapshot(workspace)
        .unwrap()
        .starts
        .remove(0);
    let creates = harness.ports.creates.lock().unwrap();
    assert_eq!(creates.len(), 1);
    assert_eq!(creates[0].operation_id.as_uuid(), start.delivery_id);
    assert_eq!(creates[0].sandbox_id.as_uuid(), intent.runner_id);
    assert_eq!(intent.provider, OverflowProvider::Boat);
    assert_eq!(start.provider, OverflowProvider::Boat);
    assert_eq!(start.snapshot, "shared-image");
    assert_eq!(start.image_root_bytes, 47_000_000_000);
    assert_eq!(
        intent.required_root_disk_bytes,
        start.required_root_disk_bytes
    );
}

#[tokio::test]
async fn provider_first_label_uses_own_machine_when_refresh_fails() {
    let harness = Harness::new(true, false).await;
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;
    let mut label = store.labels(workspace).unwrap().remove(0);
    label.order.swap(0, 1);
    store.update_label(workspace, &label).unwrap();
    harness.ports.fail.store(true, Ordering::SeqCst);
    harness.queued();

    harness.runtime.settled_tick().await.unwrap();

    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 0);
}

#[tokio::test]
async fn machine_only_job_starts_while_provider_first_job_waits_for_refresh() {
    let harness = Harness::new(true, false).await;
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;
    let mut label = store.labels(workspace).unwrap().remove(0);
    label.order.swap(0, 1);
    store.update_label(workspace, &label).unwrap();
    let provider_job = harness.queued_at(at() - Duration::seconds(1));
    label.id = Uuid::new_v4();
    label.name = "machine-only".to_owned();
    label.order = vec![LabelPlace::Machine];
    store.create_label(workspace, &label).unwrap();
    let binding = ScaleSet {
        id: Uuid::new_v4(),
        organization_id: harness.fixture.organization,
        label_id: label.id,
        github_id: 60,
        runner_group_id: 45,
    };
    store.create_scale_set(workspace, &binding).unwrap();
    harness.queued_for(label.id, binding.id);

    harness.runtime.settled_tick().await.unwrap();

    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 1);
    let intent = store.overflow_runner_intents(workspace).unwrap().remove(0);
    assert_eq!(intent.label_id, provider_job.label_id);
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn both_exit_paths_share_the_one_use_claim() {
    let harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    let input = harness.ports.inputs.lock().unwrap()[0].clone();
    let token = input["exit_report_token"].as_str().unwrap();
    let runner = input["configure"]["runner_id"].as_str().unwrap();
    let router = overflow_callback::router(harness.service.clone());
    let neutral = "/hooks/overflow/runner-exit";
    let legacy = "/hooks/boat/runner-exit";
    assert_eq!(
        input["exit_report_url"],
        "https://app.ciproxy.example/hooks/overflow/runner-exit"
    );
    assert_eq!(
        post_exit_at(&router, neutral, token, runner).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        post_exit_at(&router, legacy, token, runner).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 1);
    let start = harness
        .fixture
        .store
        .overflow_runtime_snapshot(harness.fixture.workspace)
        .unwrap()
        .starts
        .remove(0);
    assert_eq!(start.phase, StartPhase::Ended);
}
