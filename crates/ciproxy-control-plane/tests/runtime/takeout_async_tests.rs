//! Production take-out callbacks require a current saved Busy or retirement answer.

use std::sync::atomic::Ordering;

use chrono::Duration;
use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_github::registrations::RegistrationRetirement;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::runners::RunnerState;
use rust_decimal::Decimal;

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

#[tokio::test]
async fn takeout_busy_proof_survives_cost_refresh_during_and_after_its_remote_check() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
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
    harness.runtime.overflow = None;
    let pending = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(pending, Err(Error::StateChanged)), "{pending:?}");
    remote.entered.notified().await;
    *harness.ports.now.lock().unwrap() = at() + Duration::milliseconds(500);
    *harness.ports.usage_usd.lock().unwrap() = Some(Decimal::new(1, 2));
    harness.service.view(workspace).await.unwrap();
    let during = harness
        .fixture
        .store
        .own_routing_snapshot(workspace)
        .unwrap();
    assert_eq!(
        during.overflow.starts[0]
            .runtime_total
            .as_ref()
            .unwrap()
            .usd,
        Decimal::new(1, 2)
    );
    assert_eq!(
        during.overflow.starts[0].starts_at,
        at() - Duration::milliseconds(500)
    );
    remote.release.add_permits(1);
    let written = harness.runtime.quiesce().await;
    assert!(written.is_ok(), "{written:?}");
    *harness.ports.now.lock().unwrap() = at() + Duration::seconds(2);
    *harness.ports.usage_usd.lock().unwrap() = Some(Decimal::new(2, 2));
    harness.service.view(workspace).await.unwrap();
    let after = harness
        .fixture
        .store
        .own_routing_snapshot(workspace)
        .unwrap();
    assert_eq!(
        after.overflow.starts[0].starts_at,
        at() + Duration::seconds(1)
    );
    assert_eq!(
        after.overflow.starts[0].runtime_total.as_ref().unwrap().usd,
        Decimal::new(2, 2)
    );
    assert_eq!(after.overflow.starts[0].ends_at, None);
    harness.runtime.lifecycle(event.clone()).await.unwrap();
    harness.runtime.quiesce_lifecycle(&event).await.unwrap();
    assert_eq!(*remote.retirements.lock().unwrap(), vec![Some(100)]);
}
