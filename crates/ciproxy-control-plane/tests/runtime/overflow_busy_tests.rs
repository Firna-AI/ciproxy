//! Saved GitHub busy answers do not cover another waiting job.

use std::sync::atomic::Ordering;

use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::runners::RunnerState;

use super::overflow_support::Harness;
use super::settled::Settled;
use super::support::at;

#[tokio::test]
async fn github_busy_boat_runner_does_not_cover_new_waiting_demand() {
    let harness = Harness::new(false, false).await;
    let mut completed = harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    completed.phase = DemandPhase::Completed;
    completed.completed_at = Some(at());
    harness
        .fixture
        .store
        .save_github_job(harness.fixture.workspace, &completed, at())
        .unwrap();
    harness.busy.store(true, Ordering::SeqCst);
    harness.runtime.settled_tick().await.unwrap();
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    assert_eq!(
        harness
            .fixture
            .store
            .runners(harness.fixture.workspace, harness.fixture.organization)
            .unwrap()
            .into_iter()
            .find(|runner| runner.github_id == Some(100))
            .unwrap()
            .state,
        RunnerState::Busy
    );
    harness.queued();
    harness
        .fixture
        .store
        .set_machine_draining(harness.fixture.workspace, harness.fixture.machine, false)
        .unwrap();
    harness.runtime.settled_tick().await.unwrap();
    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 1);
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn absent_boat_service_keeps_own_capacity_available_after_saved_boat_supply() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 1);
    harness.runtime.overflow = None;
    harness
        .fixture
        .store
        .set_machine_draining(harness.fixture.workspace, harness.fixture.machine, false)
        .unwrap();
    harness.runtime.settled_tick().await.unwrap();
    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 1);
}
