//! HTTP callbacks derive tenant authority only from the saved sandbox token.

use std::sync::atomic::Ordering;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Duration;
use ciproxy_control_plane::boat_callback;
use ciproxy_github::registrations::RegistrationRetirement;
use ciproxy_overflow::api::StartRequest;
use ciproxy_overflow::error::Error;
use ciproxy_protocol::identifiers::{LabelName, OrganizationId};
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Destination, StartReason, StartRunner};
use ciproxy_scaler::boat::BoatSizeId;
use ciproxy_scaler::registration::RegistrationTarget;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::workspaces::Workspace;
use tower::ServiceExt;
use uuid::Uuid;

use super::held_remote::HeldRemote;
use super::overflow_support::Harness;
use super::settled::Settled;
use super::support::at;

async fn post_exit(router: &Router, token: &str, runner: &str) -> StatusCode {
    let request = Request::builder()
        .method("POST")
        .uri("/hooks/boat/runner-exit")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({"type":"runner_exited","runner_id":runner,"exit":{"code":0}})
                .to_string(),
        ))
        .unwrap();
    router.clone().oneshot(request).await.unwrap().status()
}

#[tokio::test]
async fn callback_tokens_cannot_end_a_runner_in_another_workspace() {
    let harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    let other = Uuid::new_v4();
    let runner = Uuid::new_v4();
    let store = &harness.fixture.store;
    store
        .create_workspace(&Workspace {
            id: other,
            name: "Second customer".to_owned(),
            delete_after: None,
        })
        .unwrap();
    let mut organization = store
        .organization(harness.fixture.workspace, harness.fixture.organization)
        .unwrap()
        .unwrap();
    organization.id = Uuid::new_v4();
    organization.workspace_id = other;
    organization.github_id = 90;
    organization.installation_id = 91;
    store.install_organization(&organization).unwrap();
    let mut label = store.labels(harness.fixture.workspace).unwrap().remove(0);
    label.id = Uuid::new_v4();
    store.create_label(other, &label).unwrap();
    let set = ScaleSet {
        id: Uuid::new_v4(),
        organization_id: organization.id,
        label_id: label.id,
        github_id: 92,
        runner_group_id: 93,
    };
    store.create_scale_set(other, &set).unwrap();
    harness
        .service
        .connect(
            other,
            Uuid::new_v4(),
            Secret::new("other-fake-key"),
            "personal".to_owned(),
            200_000_000,
        )
        .await
        .unwrap();
    harness
        .service
        .start(
            other,
            StartRequest {
                delivery_id: Uuid::new_v4(),
                runner_id: runner,
                organization_id: organization.id,
                label_id: label.id,
                scale_set_id: set.id,
                action: StartRunner {
                    platform: Platform::LinuxX64,
                    target: RegistrationTarget {
                        organization_id: OrganizationId(organization.id.to_string()),
                        label_id: label.id.to_string(),
                        scale_set_id: set.id.to_string(),
                        github_id: set.github_id,
                    },
                    organization_id: OrganizationId("90".to_owned()),
                    label: LabelName(label.name),
                    requested_size: VmSize {
                        vcpu: 2,
                        memory_bytes: 8_000_000_000,
                        disk_bytes: 20_000_000_000,
                    },
                    size: VmSize {
                        vcpu: 8,
                        memory_bytes: 16_000_000_000,
                        disk_bytes: 111_000_000_000,
                    },
                    destination: Destination::Boat(BoatSizeId("large".to_owned())),
                    reason: StartReason::NoEligibleMachine,
                },
                registration: Secret::new("other-fake-jit"),
            },
        )
        .await
        .unwrap();
    let inputs = harness.ports.inputs.lock().unwrap().clone();
    let first_token = inputs[0]["exit_report_token"].as_str().unwrap();
    let other_token = inputs[1]["exit_report_token"].as_str().unwrap();
    let first_runner = inputs[0]["configure"]["runner_id"].as_str().unwrap();
    let router = boat_callback::router(harness.service.clone());
    assert_eq!(
        post_exit(&router, first_token, "not-a-uuid").await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        post_exit(&router, first_token, &runner.to_string()).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post_exit(&router, other_token, first_runner).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        store.boat_runtime_snapshot(other).unwrap().starts[0].ends_at,
        None
    );
    assert_eq!(
        post_exit(&router, first_token, first_runner).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        store.boat_runtime_snapshot(other).unwrap().starts[0].ends_at,
        None
    );
    assert_eq!(
        post_exit(&router, other_token, &runner.to_string()).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn callback_saves_exit_while_workspace_lock_remains_held() {
    let mut harness = Harness::new(false, false).await;
    let mut job = harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    let workspace = harness.fixture.workspace;
    let remote = HeldRemote::new(
        workspace,
        false,
        RegistrationRetirement::Removed { id: 100 },
    );
    harness.use_registrations(remote.clone());
    let store = &harness.fixture.store;
    job.phase = DemandPhase::Completed;
    job.runner_github_id = Some(100);
    job.completed_at = Some(at());
    store.save_github_job(workspace, &job, at()).unwrap();
    let inputs = harness.ports.inputs.lock().unwrap().clone();
    let token = inputs[0]["exit_report_token"].as_str().unwrap();
    let runner = inputs[0]["configure"]["runner_id"].as_str().unwrap();
    let router = boat_callback::router(harness.service.clone());
    let reported_at = *harness.ports.now.lock().unwrap();
    let cleanup = harness.service.cleanup(workspace);
    tokio::pin!(cleanup);
    tokio::select! {
        () = remote.entered.notified() => {},
        result = &mut cleanup => panic!("cleanup did not reach GitHub retirement: {result:?}"),
    }

    assert_eq!(
        post_exit(&router, token, runner).await,
        StatusCode::NO_CONTENT
    );
    let claim = store
        .boat_runtime_snapshot(workspace)
        .unwrap()
        .starts
        .remove(0);
    assert_eq!(claim.reported_at, Some(reported_at));
    assert_eq!(claim.phase, StartPhase::Deleting);
    assert_eq!(claim.ends_at, None);
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    assert_eq!(
        post_exit(&router, token, runner).await,
        StatusCode::UNAUTHORIZED
    );
    remote.release.add_permits(1);
    assert!(matches!(cleanup.await, Err(Error::ProviderUnavailable)));
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    assert_eq!(
        store.boat_runtime_snapshot(workspace).unwrap().starts,
        vec![claim]
    );

    let ended_at = reported_at + Duration::minutes(1);
    *harness.ports.now.lock().unwrap() = ended_at;
    harness.service.cleanup(workspace).await.unwrap();
    let ended = store
        .boat_runtime_snapshot(workspace)
        .unwrap()
        .starts
        .remove(0);
    assert_eq!(ended.reported_at, Some(reported_at));
    assert_eq!(ended.phase, StartPhase::Ended);
    assert_eq!(ended.ends_at, Some(ended_at));
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(*remote.retirements.lock().unwrap(), vec![Some(100)]);
}

#[tokio::test]
async fn callback_returns_503_for_claim_failure_and_204_for_delete_failure() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    let workspace = harness.fixture.workspace;
    let store = harness.fixture.store.clone();
    let inputs = harness.ports.inputs.lock().unwrap().clone();
    let token = inputs[0]["exit_report_token"].as_str().unwrap();
    let runner = inputs[0]["configure"]["runner_id"].as_str().unwrap();
    let router = boat_callback::router(harness.service.clone());
    let reported_at = *harness.ports.now.lock().unwrap();
    let before = store.boat_runtime_snapshot(workspace).unwrap().starts;

    harness.fixture.reject_boat_exit_claims();
    assert_eq!(
        post_exit(&router, token, runner).await,
        StatusCode::SERVICE_UNAVAILABLE,
        "a failed durable claim must return HTTP 503"
    );
    assert_eq!(
        store.boat_runtime_snapshot(workspace).unwrap().starts,
        before
    );
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    harness.fixture.allow_boat_exit_claims();

    harness.ports.delete_fail.store(true, Ordering::SeqCst);

    assert_eq!(
        post_exit(&router, token, runner).await,
        StatusCode::NO_CONTENT
    );
    let claim = store
        .boat_runtime_snapshot(workspace)
        .unwrap()
        .starts
        .remove(0);
    assert_eq!(claim.reported_at, Some(reported_at));
    assert_eq!(claim.phase, StartPhase::Deleting);
    assert_eq!(claim.ends_at, None);
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(
        post_exit(&router, token, runner).await,
        StatusCode::UNAUTHORIZED
    );

    harness.ports.delete_fail.store(false, Ordering::SeqCst);
    harness.service.cleanup(workspace).await.unwrap();
    let ended = store
        .boat_runtime_snapshot(workspace)
        .unwrap()
        .starts
        .remove(0);
    assert_eq!(ended.reported_at, Some(reported_at));
    assert_eq!(ended.phase, StartPhase::Ended);
    assert_eq!(ended.ends_at, Some(reported_at));
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 2);
}
