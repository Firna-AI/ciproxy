//! HTTP callbacks derive tenant authority only from the saved sandbox token.

use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ciproxy_control_plane::boat_callback;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_overflow::api::StartRequest;
use ciproxy_protocol::identifiers::{LabelName, OrganizationId};
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Destination, StartReason, StartRunner};
use ciproxy_scaler::boat::BoatSizeId;
use ciproxy_scaler::registration::RegistrationTarget;
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::workspaces::Workspace;
use tower::ServiceExt;
use uuid::Uuid;

use super::overflow_support::Harness;

#[tokio::test]
async fn callback_tokens_cannot_end_a_runner_in_another_workspace() {
    let harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.tick().await.unwrap();
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
    let request = |token: &str, runner: &str| {
        Request::builder()
            .method("POST")
            .uri("/hooks/boat/runner-exit")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"type":"runner_exited","runner_id":runner,"exit":{"code":0}})
                    .to_string(),
            ))
            .unwrap()
    };
    assert_eq!(
        router
            .clone()
            .oneshot(request(first_token, &runner.to_string()))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        router
            .clone()
            .oneshot(request(other_token, first_runner))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        store.boat_runtime_snapshot(other).unwrap().starts[0].ends_at,
        None
    );
    assert_eq!(
        router
            .clone()
            .oneshot(request(first_token, first_runner))
            .await
            .unwrap()
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        store.boat_runtime_snapshot(other).unwrap().starts[0].ends_at,
        None
    );
    assert_eq!(
        router
            .oneshot(request(other_token, &runner.to_string()))
            .await
            .unwrap()
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 2);
}
