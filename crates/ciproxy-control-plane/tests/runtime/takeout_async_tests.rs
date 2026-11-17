//! Production take-out callbacks require a current saved Busy or retirement answer.

use std::sync::atomic::Ordering;

use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_github::registrations::RegistrationRetirement;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::runners::RunnerState;

use super::held_remote::HeldRemote;
use super::overflow_support::Harness;
use super::overflow_takeout_support::prepare;
use super::settled::Settled;
use super::support::at;
use super::takeout_support::{Answer, Remote};

#[tokio::test]
async fn takeout_callback_waits_for_current_busy_answer_and_rechecks_changed_saved_evidence() {
    let mut harness = Harness::new(false, false).await;
    let mut job = harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    let workspace = harness.fixture.workspace;
    let organization = harness.fixture.organization;
    let intent = harness
        .fixture
        .store
        .boat_runner_intents(workspace)
        .unwrap()
        .remove(0);
    harness
        .fixture
        .store
        .update_runner(
            workspace,
            organization,
            intent.runner_id,
            RunnerState::Busy,
            Some(100),
        )
        .unwrap();
    let event = prepare(&mut harness, Remote::new(Answer::Busy));
    let remote = HeldRemote::new(workspace, false, RegistrationRetirement::Busy { id: 100 });
    harness.use_registrations(remote.clone());
    let pending = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(pending, Err(Error::StateChanged)), "{pending:?}");
    remote.entered.notified().await;
    let held = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(held, Err(Error::StateChanged)), "{held:?}");
    assert_eq!(
        harness
            .fixture
            .store
            .pending_github_lifecycle(42)
            .unwrap()
            .len(),
        1
    );
    remote.release.add_permits(10);
    harness.runtime.quiesce().await.unwrap();
    harness.runtime.lifecycle(event.clone()).await.unwrap();
    harness.runtime.quiesce_lifecycle(&event).await.unwrap();
    job.phase = DemandPhase::Completed;
    job.runner_github_id = Some(100);
    job.completed_at = Some(at());
    harness
        .fixture
        .store
        .save_github_job(workspace, &job, at())
        .unwrap();
    remote.fail.store(true, Ordering::SeqCst);
    let changed = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(changed, Err(Error::StateChanged)), "{changed:?}");
    let failed = harness.runtime.quiesce().await;
    assert!(matches!(failed, Err(Error::StateChanged)), "{failed:?}");
    let unavailable = harness.runtime.lifecycle(event).await;
    assert!(
        matches!(unavailable, Err(Error::StateChanged)),
        "{unavailable:?}"
    );
    let failed = harness.runtime.quiesce().await;
    assert!(matches!(failed, Err(Error::StateChanged)), "{failed:?}");
    assert_eq!(
        harness
            .fixture
            .store
            .runners(workspace, organization)
            .unwrap()
            .into_iter()
            .find(|runner| runner.id == intent.runner_id)
            .unwrap()
            .state,
        RunnerState::Busy
    );
    assert_eq!(
        harness
            .fixture
            .store
            .pending_github_lifecycle(42)
            .unwrap()
            .len(),
        1
    );
}
