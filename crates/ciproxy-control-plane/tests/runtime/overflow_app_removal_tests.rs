//! App removal retains launched Boat guests until their exit or provider lifetime.

use std::sync::atomic::Ordering;

use chrono::Duration;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_overflow::error::Error;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::runners::RunnerState;
use sandbox_interface::SandboxLifetime;

use super::overflow_support::Harness;
use super::support::at;

#[tokio::test]
async fn removed_app_retains_idle_and_running_boat_supply_to_the_full_or_trial_lifetime() {
    for (trial, lifetime_seconds) in [(false, 21_600), (true, 7_200)] {
        let harness = Harness::new(false, false).await;
        harness.ports.trial.store(trial, Ordering::SeqCst);
        let mut idle = harness.queued();
        let mut running = harness.queued();
        harness.runtime.tick().await.unwrap();
        assert_eq!(harness.ports.launches.load(Ordering::SeqCst), 2);
        for create in harness.ports.creates.lock().unwrap().iter() {
            assert_eq!(
                create.lifetime,
                SandboxLifetime::OneShot {
                    max_lifetime: std::time::Duration::from_secs(lifetime_seconds),
                },
                "trial: {trial}"
            );
        }
        idle.phase = DemandPhase::Completed;
        idle.completed_at = Some(at());
        running.phase = DemandPhase::Started;
        running.started_at = Some(at());
        running.runner_github_id = Some(100);
        let store = &harness.fixture.store;
        let workspace = harness.fixture.workspace;
        let organization = harness.fixture.organization;
        store.save_github_job(workspace, &idle, at()).unwrap();
        store.save_github_job(workspace, &running, at()).unwrap();
        harness.queued();
        store
            .record_app_removal(workspace, organization, at())
            .unwrap();
        harness.runtime.tick().await.unwrap();
        assert_eq!(
            harness.ports.deletes.load(Ordering::SeqCst),
            0,
            "trial: {trial}"
        );
        assert_eq!(harness.ports.creates.lock().unwrap().len(), 2);
        assert_eq!(harness.own_starts.load(Ordering::SeqCst), 0);
        let runners = store.runners(workspace, organization).unwrap();
        assert_eq!(
            runners
                .iter()
                .filter(|runner| runner.state == RunnerState::Busy)
                .count(),
            1
        );
        let live = store.boat_runtime_snapshot(workspace).unwrap();
        assert_eq!(live.accounts.len(), 1);
        assert_eq!(live.accounts[0].account.retired_at, Some(at()));
        assert_eq!(
            live.starts
                .iter()
                .map(|record| (record.phase, record.ends_at))
                .collect::<Vec<_>>(),
            vec![(StartPhase::Running, None); 2]
        );
        let end = at() + Duration::seconds(lifetime_seconds.try_into().unwrap());
        *harness.ports.now.lock().unwrap() = end - Duration::seconds(1);
        harness.runtime.tick().await.unwrap();
        assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
        assert_eq!(harness.ports.creates.lock().unwrap().len(), 2);
        harness.ports.fail.store(true, Ordering::SeqCst);
        assert!(matches!(
            harness.service.cleanup(workspace).await,
            Err(Error::ProviderUnavailable)
        ));
        let before = store.boat_runtime_snapshot(workspace).unwrap();
        assert_eq!(before.accounts.len(), 1);
        assert_eq!(
            before
                .starts
                .iter()
                .map(|record| record.ends_at)
                .collect::<Vec<_>>(),
            vec![None; 2]
        );
        *harness.ports.now.lock().unwrap() = end;
        harness.service.cleanup(workspace).await.unwrap();
        let expired = store.boat_runtime_snapshot(workspace).unwrap();
        assert_eq!(expired.accounts.len(), 0);
        assert_eq!(
            expired
                .starts
                .iter()
                .map(|record| (record.phase, record.ends_at))
                .collect::<Vec<_>>(),
            vec![(StartPhase::Ended, Some(end)); 2]
        );
        assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn app_removal_during_github_retirement_retains_the_launched_sandbox() {
    let harness = Harness::new(false, false).await;
    let mut job = harness.queued();
    harness.runtime.tick().await.unwrap();
    job.phase = DemandPhase::Completed;
    job.completed_at = Some(at());
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;
    store.save_github_job(workspace, &job, at()).unwrap();
    harness.remove_app_on_retire.store(true, Ordering::SeqCst);
    let result = harness.service.cleanup(workspace).await;
    assert_eq!(
        store
            .organization(workspace, harness.fixture.organization)
            .unwrap()
            .unwrap()
            .removed_at,
        Some(at())
    );
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    assert!(
        matches!(result, Err(Error::ProviderUnavailable)),
        "{result:?}"
    );
    let retained = store.boat_runtime_snapshot(workspace).unwrap();
    assert_eq!(retained.starts[0].phase, StartPhase::Running);
    assert_eq!(retained.starts[0].ends_at, None);
    assert_eq!(retained.accounts.len(), 1);
}
