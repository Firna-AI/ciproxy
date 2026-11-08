//! Idle cleanup follows the scaler's shared Ready-before-Starting priority.

use std::sync::atomic::Ordering;

use chrono::Duration;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use uuid::Uuid;

use super::overflow_support::Harness;
use super::support::at;

#[tokio::test]
async fn ready_boat_supply_survives_starting_own_supply_and_an_older_missing_delivery() {
    let harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.tick().await.unwrap();
    let workspace = harness.fixture.workspace;
    let store = &harness.fixture.store;
    store
        .set_machine_draining(workspace, harness.fixture.machine, false)
        .unwrap();
    let label = store.labels(workspace).unwrap().remove(0);
    let live = store.boat_runner_intents(workspace).unwrap().remove(0);
    let mut missing = live.clone();
    missing.runner_id = Uuid::new_v4();
    missing.delivery_id = Uuid::new_v4();
    missing.created_at = at() - Duration::seconds(10);
    missing.github_id = Some(99);
    store
        .create_boat_runner_intent(workspace, &missing)
        .unwrap();
    store
        .create_own_runner(
            workspace,
            &OwnRunner {
                registration_scale_set_id: Some(44),
                requested_size: label.size,
                slot_indices: vec![0],
                runner: Runner {
                    id: Uuid::new_v4(),
                    organization_id: harness.fixture.organization,
                    label_id: label.id,
                    scale_set_id: harness.fixture.intent.runner.scale_set_id,
                    github_id: None,
                    state: RunnerState::Starting,
                    size: ciproxy_protocol::resources::VmSize {
                        vcpu: 2,
                        memory_bytes: 8_000_000_000,
                        disk_bytes: 80_899_612_672,
                    },
                    place: RunnerPlace::Machine {
                        machine_id: harness.fixture.machine,
                        first_slot: 0,
                    },
                },
                label_name: label.name,
                command_id: Uuid::new_v4(),
                issued_at: at(),
                acknowledged: false,
                confirmed_at: None,
            },
        )
        .unwrap();
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    assert_eq!(
        store.boat_runtime_snapshot(workspace).unwrap().starts[0].ends_at,
        None
    );
    assert_eq!(store.boat_runner_intents(workspace).unwrap().len(), 1);
}

#[tokio::test]
async fn failed_completed_boat_cleanup_does_not_block_a_new_own_start() {
    let harness = Harness::new(false, false).await;
    let mut completed = harness.queued();
    harness.runtime.tick().await.unwrap();
    completed.phase = DemandPhase::Completed;
    completed.runner_github_id = Some(100);
    completed.completed_at = Some(at());
    harness
        .fixture
        .store
        .save_github_job(harness.fixture.workspace, &completed, at())
        .unwrap();
    harness
        .fixture
        .store
        .set_machine_draining(harness.fixture.workspace, harness.fixture.machine, false)
        .unwrap();
    harness.queued();
    harness.ports.fail.store(true, Ordering::SeqCst);
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    assert_eq!(
        harness
            .fixture
            .store
            .boat_runtime_snapshot(harness.fixture.workspace)
            .unwrap()
            .starts[0]
            .ends_at,
        None
    );
}
