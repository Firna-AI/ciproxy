//! A new runtime resumes a durable post-create delivery without duplicate cost.

use std::slice;
use std::sync::atomic::Ordering;

use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::runners::RunnerPlace;
use rust_decimal::Decimal;
use sandbox_interface::{BackendSandbox, ProviderRef, SandboxState};

use super::overflow_service::{authority, service};
use super::overflow_support::Harness;

#[tokio::test]
async fn runtime_restart_recovers_post_create_delivery_with_same_identities_and_one_cost_bound() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
    harness.ports.pending.store(true, Ordering::SeqCst);
    harness.runtime.tick().await.unwrap();
    let workspace = harness.fixture.workspace;
    let store = harness.fixture.store.clone();
    let pending = store.boat_runtime_snapshot(workspace).unwrap();
    assert_eq!(pending.starts.len(), 1);
    let original = &pending.starts[0];
    assert_eq!(original.phase, StartPhase::Creating);
    assert_eq!(original.sandbox, None);
    assert_eq!(harness.ports.launches.load(Ordering::SeqCst), 0);
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 1);
    let request = harness.ports.creates.lock().unwrap()[0].clone();
    assert_eq!(request.operation_id.as_uuid(), original.delivery_id);
    assert_eq!(request.sandbox_id.as_uuid(), original.runner_id);
    let intents = store.boat_runner_intents(workspace).unwrap();
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[0].delivery_id, original.delivery_id);
    assert_eq!(intents[0].runner_id, original.runner_id);
    assert_eq!(intents[0].github_id, Some(100));
    let expected_bound = Decimal::new(502, 3);
    let before = harness.service.view(workspace).await.unwrap().spend;
    assert_eq!(before.confirmed_usd, Decimal::ZERO);
    assert_eq!(before.pending_usd, expected_bound);
    assert_eq!(before.settled_bound_usd, Decimal::ZERO);
    assert_eq!(before.counted_usd().unwrap(), expected_bound);

    let provider = ProviderRef::new("recovered-runtime-sandbox");
    *harness.ports.recovered.lock().unwrap() = Some(BackendSandbox {
        provider_ref: provider.clone(),
        state: SandboxState::Ready,
    });
    let registrations = harness.runtime.registrations.clone();
    harness.service = service(&harness.fixture, &harness.ports, registrations.clone());
    let mut restarted = MachineRuntime::new(
        store.clone(),
        harness.runtime.gateway.clone(),
        registrations.clone(),
        harness.runtime.assignments.clone(),
        harness.ports.clone(),
        harness.runtime.image.clone(),
        harness.runtime.installations.clone(),
    );
    restarted.overflow = Some(harness.service.clone());
    restarted.boat_retirement = Some(authority(&harness.fixture, &harness.ports, registrations));
    harness.runtime = restarted;

    for _ in 0..2 {
        harness.runtime.tick().await.unwrap();
        let recovered = store.boat_runtime_snapshot(workspace).unwrap();
        assert_eq!(
            harness.ports.launches.load(Ordering::SeqCst),
            1,
            "post-create delivery must launch after restart: {:?}",
            recovered.starts
        );
        assert_eq!(
            *harness.ports.launch_targets.lock().unwrap(),
            [(provider.clone(), original.delivery_id)]
        );
        assert_eq!(recovered.starts.len(), 1);
        let record = &recovered.starts[0];
        assert_eq!(record.phase, StartPhase::Running);
        assert_eq!(record.delivery_id, original.delivery_id);
        assert_eq!(record.runner_id, original.runner_id);
        assert_eq!(record.organization_id, original.organization_id);
        assert_eq!(record.label_id, original.label_id);
        assert_eq!(record.scale_set_id, original.scale_set_id);
        assert_eq!(record.key_id, original.key_id);
        assert_eq!(record.dispatch, original.dispatch);
        assert_eq!(record.starts_at, original.starts_at);
        assert_eq!(record.sandbox.as_ref().unwrap().id(), provider.as_str());
        assert_eq!(store.boat_runner_intents(workspace).unwrap(), intents);
        let runners: Vec<_> = store
            .runners(workspace, harness.fixture.organization)
            .unwrap()
            .into_iter()
            .filter(|runner| matches!(runner.place, RunnerPlace::Boat { .. }))
            .collect();
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].id, original.runner_id);
        assert_eq!(runners[0].github_id, intents[0].github_id);
        assert_eq!(
            runners[0].place,
            RunnerPlace::Boat {
                key_id: Some(original.key_id),
                sandbox_id: provider.as_str().to_owned(),
                size_name: "large".to_owned(),
            }
        );
        let inputs = harness.ports.inputs.lock().unwrap().clone();
        assert_eq!(inputs.len(), 1);
        assert_eq!(
            inputs[0]["configure"]["runner_id"],
            original.runner_id.to_string()
        );
        assert_eq!(inputs[0]["configure"]["registration"], "fake-jit");
        assert_eq!(
            harness.ports.creates.lock().unwrap().as_slice(),
            slice::from_ref(&request)
        );
        assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
        assert_eq!(harness.own_starts.load(Ordering::SeqCst), 0);
        let after = harness.service.view(workspace).await.unwrap().spend;
        assert_eq!(after.confirmed_usd, Decimal::ZERO);
        assert_eq!(after.pending_usd, expected_bound);
        assert_eq!(after.settled_bound_usd, Decimal::ZERO);
        assert_eq!(after.counted_usd().unwrap(), expected_bound);
    }
}
