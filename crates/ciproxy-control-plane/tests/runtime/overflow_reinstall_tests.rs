//! Reinstallation restores early idle cleanup with old retained Boat keys.

use std::sync::atomic::Ordering;

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::runners::RunnerState;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::support::at;

#[tokio::test]
async fn reinstall_then_boat_reconnect_retires_old_idle_supply_and_preserves_running_work() {
    let harness = Harness::new(false, false).await;
    let mut idle = harness.queued();
    let mut running = harness.queued();
    harness.runtime.tick().await.unwrap();
    let workspace = harness.fixture.workspace;
    let organization = harness.fixture.organization;
    let store = &harness.fixture.store;
    idle.phase = DemandPhase::Completed;
    idle.completed_at = Some(at());
    running.phase = DemandPhase::Started;
    running.started_at = Some(at());
    running.runner_github_id = Some(101);
    store.save_github_job(workspace, &idle, at()).unwrap();
    store.save_github_job(workspace, &running, at()).unwrap();
    store
        .record_app_removal(workspace, organization, at())
        .unwrap();
    harness.runtime.tick().await.unwrap();
    let removed = store.boat_runtime_snapshot(workspace).unwrap();
    let old_key = removed.accounts[0].account.id;
    assert_eq!(removed.accounts[0].account.retired_at, Some(at()));
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    let mut reinstalled = store
        .organization(workspace, organization)
        .unwrap()
        .unwrap();
    reinstalled.installation_id = 49;
    store.install_organization(&reinstalled).unwrap();
    assert_eq!(
        store
            .organization(workspace, organization)
            .unwrap()
            .unwrap()
            .removed_at,
        None
    );
    let result = harness.service.cleanup(workspace).await;
    assert_eq!(
        harness.ports.deletes.load(Ordering::SeqCst),
        1,
        "{result:?}"
    );
    result.unwrap();
    let before_reconnect = store.boat_runtime_snapshot(workspace).unwrap();
    assert_eq!(
        before_reconnect
            .starts
            .iter()
            .filter(|record| record.phase == StartPhase::Ended
                && record.ends_at == Some(at())
                && record.key_id == old_key)
            .count(),
        1
    );
    assert_eq!(
        before_reconnect
            .starts
            .iter()
            .filter(|record| record.phase == StartPhase::Running
                && record.ends_at.is_none()
                && record.key_id == old_key)
            .count(),
        1
    );
    let new_key = Uuid::new_v4();
    harness
        .service
        .connect(
            workspace,
            new_key,
            Secret::new("fake-reconnected-boat-key"),
            "personal".to_owned(),
            200_000_000,
        )
        .await
        .unwrap();
    harness.runtime.tick().await.unwrap();
    let reconnected = store.boat_runtime_snapshot(workspace).unwrap();
    assert_eq!(reconnected.accounts.len(), 2);
    assert_eq!(
        reconnected
            .accounts
            .iter()
            .find(|account| account.account.id == new_key)
            .unwrap()
            .account
            .retired_at,
        None
    );
    assert_eq!(
        reconnected
            .starts
            .iter()
            .filter(|record| record.phase == StartPhase::Running
                && record.ends_at.is_none()
                && record.key_id == old_key)
            .count(),
        1
    );
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.launches.load(Ordering::SeqCst), 2);
    assert_eq!(
        store
            .runners(workspace, organization)
            .unwrap()
            .iter()
            .filter(|runner| runner.state == RunnerState::Busy && runner.github_id == Some(101))
            .count(),
        1
    );
    assert_eq!(
        store
            .github_jobs(workspace, organization)
            .unwrap()
            .iter()
            .find(|job| job.id == running.id)
            .unwrap()
            .phase,
        DemandPhase::Started
    );
}
