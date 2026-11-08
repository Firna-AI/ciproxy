//! Same-name labels cannot reuse registrations from their retained old binding.

use std::sync::atomic::Ordering;

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::support::at;

async fn replacement(state: RunnerState) {
    let harness = Harness::new(true, false).await;
    let fixture = &harness.fixture;
    let store = &fixture.store;
    let mut contact = fixture.contact.clone();
    contact.report.runners.clear();
    contact.report.capacity = Slots(3);
    store
        .record_contact(fixture.workspace, fixture.machine, &contact)
        .unwrap();
    let mut label = store.labels(fixture.workspace).unwrap().remove(0);
    label.id = Uuid::new_v4();
    label.name = "own-replacement".to_owned();
    label.allow_boat = false;
    store.create_label(fixture.workspace, &label).unwrap();
    let mut binding = ScaleSet {
        id: Uuid::new_v4(),
        organization_id: fixture.organization,
        label_id: label.id,
        github_id: 60,
        runner_group_id: 45,
    };
    store.create_scale_set(fixture.workspace, &binding).unwrap();
    harness.queued_for(label.id, binding.id);
    harness.runtime.tick().await.unwrap();
    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 1);
    let old = store
        .own_runners(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|runner| runner.runner.label_id == label.id)
        .unwrap();
    store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            old.runner.id,
            state,
            old.runner.github_id,
        )
        .unwrap();
    store
        .retire_dashboard_label(fixture.workspace, label.id, at())
        .unwrap();
    label.id = Uuid::new_v4();
    binding.id = Uuid::new_v4();
    binding.label_id = label.id;
    binding.github_id = 61;
    store.create_label(fixture.workspace, &label).unwrap();
    store.create_scale_set(fixture.workspace, &binding).unwrap();
    harness.queued_for(label.id, binding.id);
    harness.runtime.tick().await.unwrap();
    assert_eq!(
        harness.own_starts.load(Ordering::SeqCst),
        2,
        "replacement demand needs a runner registered to the replacement binding"
    );
    let runners = store.own_runners(fixture.workspace).unwrap();
    assert_eq!(
        runners
            .iter()
            .find(|runner| runner.runner.id == old.runner.id)
            .unwrap()
            .runner
            .state,
        if state == RunnerState::Busy {
            RunnerState::Busy
        } else {
            RunnerState::Stopping
        }
    );
    assert_eq!(
        runners
            .iter()
            .filter(|runner| runner.runner.scale_set_id == binding.id)
            .count(),
        1
    );
}

#[tokio::test]
async fn replacement_label_stops_old_own_supply_and_starts_its_exact_binding() {
    for state in [RunnerState::Starting, RunnerState::Ready, RunnerState::Busy] {
        replacement(state).await;
    }
}
