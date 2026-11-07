//! A replacement label gets its own Boat supply while older running work survives.

use std::sync::atomic::Ordering;

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::support::at;

#[tokio::test]
async fn replacement_label_gets_new_supply_for_ready_starting_and_running_old_bindings() {
    for (scenario, creating, running, deleted, ended) in [
        ("ready", false, false, 1, Some(at())),
        ("starting", true, false, 0, None),
        ("running", false, true, 0, None),
    ] {
        let harness = Harness::new(false, false).await;
        let store = &harness.fixture.store;
        let workspace = harness.fixture.workspace;
        let organization = harness.fixture.organization;
        let mut label = store.labels(workspace).unwrap().remove(0);
        label.id = Uuid::new_v4();
        label.name = "replacement-label".to_owned();
        store.create_label(workspace, &label).unwrap();
        let mut binding = ScaleSet {
            id: Uuid::new_v4(),
            organization_id: organization,
            label_id: label.id,
            github_id: 60,
            runner_group_id: 45,
        };
        store.create_scale_set(workspace, &binding).unwrap();
        let mut old_job = harness.queued_for(label.id, binding.id);
        harness.ports.pending.store(creating, Ordering::SeqCst);
        harness.runtime.tick().await.unwrap();
        assert_eq!(harness.ports.creates.lock().unwrap().len(), 1, "{scenario}");
        let old = store
            .boat_runtime_snapshot(workspace)
            .unwrap()
            .starts
            .remove(0);
        if running {
            old_job.phase = DemandPhase::Started;
            old_job.started_at = Some(at());
            old_job.runner_github_id = Some(100);
            store.save_github_job(workspace, &old_job, at()).unwrap();
        }
        store
            .retire_dashboard_label(workspace, label.id, at())
            .unwrap();
        label.id = Uuid::new_v4();
        store.create_label(workspace, &label).unwrap();
        binding.id = Uuid::new_v4();
        binding.label_id = label.id;
        binding.github_id = 61;
        store.create_scale_set(workspace, &binding).unwrap();
        harness.queued_for(label.id, binding.id);
        harness.runtime.tick().await.unwrap();
        assert_eq!(harness.ports.creates.lock().unwrap().len(), 2, "{scenario}");
        assert_eq!(
            harness.ports.deletes.load(Ordering::SeqCst),
            deleted,
            "{scenario}"
        );
        let starts = store.boat_runtime_snapshot(workspace).unwrap().starts;
        let old = starts
            .iter()
            .find(|record| record.delivery_id == old.delivery_id)
            .unwrap();
        assert_eq!(old.ends_at, ended, "{scenario}");
        assert_eq!(
            starts
                .iter()
                .filter(|record| record.label_id == label.id && record.scale_set_id == binding.id)
                .count(),
            1,
            "{scenario}"
        );
    }
}
