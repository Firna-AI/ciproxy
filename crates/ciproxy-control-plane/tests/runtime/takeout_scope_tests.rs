//! Take-out acceptance owns one lease and one immutable installation scope.

use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use super::recovery_support::{other_organization, saved};
use super::support::Fixture;
use super::takeout_gateway::Capture;
use super::takeout_installations::Probe;
use super::takeout_support::{Answer, Call, Remote, real_runtime};

#[tokio::test]
async fn takeout_lease_conflict_returns_error_before_accepting_the_event() {
    let fixture = Fixture::new();
    other_organization(&fixture);
    fixture
        .store
        .begin_organization_takeout(fixture.workspace, fixture.organization)
        .unwrap();
    let event = fixture
        .store
        .pending_github_lifecycle(42)
        .unwrap()
        .remove(0)
        .event;
    let _lease = fixture
        .store
        .try_lock_own_runners(fixture.workspace)
        .unwrap()
        .unwrap();
    let remote = Remote::new(Answer::Busy);
    let gateway = Capture::new(false);
    let runtime = real_runtime(&fixture, remote.clone(), gateway.clone());
    let result = runtime.lifecycle(event).await;
    assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
    assert_eq!(saved(&fixture).runner.state, RunnerState::Ready);
    assert_eq!(fixture.store.pending_github_lifecycle(42).unwrap().len(), 1);
    assert_eq!(*remote.calls.lock().unwrap(), Vec::<Call>::new());
    assert_eq!(gateway.messages.lock().unwrap().len(), 0);
}

#[tokio::test]
async fn takeout_busy_answer_preserves_a_sibling_organizations_warm_reservation() {
    let fixture = Fixture::new();
    let other = other_organization(&fixture);
    let set = ScaleSet {
        id: Uuid::new_v4(),
        organization_id: other.id,
        label_id: fixture.intent.runner.label_id,
        github_id: 92,
        runner_group_id: 93,
    };
    fixture
        .store
        .create_scale_set(fixture.workspace, &set)
        .unwrap();
    let mut sibling = fixture.intent.clone();
    sibling.runner.id = Uuid::new_v4();
    sibling.runner.organization_id = other.id;
    sibling.runner.scale_set_id = set.id;
    sibling.runner.github_id = Some(48);
    sibling.runner.place = RunnerPlace::Machine {
        machine_id: fixture.machine,
        first_slot: 1,
    };
    sibling.slot_indices = vec![1];
    sibling.command_id = Uuid::new_v4();
    sibling.acknowledged = false;
    fixture
        .store
        .create_own_runner(fixture.workspace, &sibling)
        .unwrap();
    fixture
        .store
        .begin_organization_takeout(fixture.workspace, fixture.organization)
        .unwrap();
    let event = fixture
        .store
        .pending_github_lifecycle(42)
        .unwrap()
        .remove(0)
        .event;
    let remote = Remote::new(Answer::Busy);
    let gateway = Capture::new(false);
    let mut runtime = real_runtime(&fixture, remote.clone(), gateway.clone());
    runtime.installations = Probe::present();
    runtime.lifecycle(event).await.unwrap();
    let rows = fixture.store.own_runners(fixture.workspace).unwrap();
    assert_eq!(
        rows.iter()
            .find(|runner| runner.runner.id == sibling.runner.id)
            .unwrap(),
        &sibling
    );
    assert_eq!(
        rows.iter()
            .find(|runner| runner.runner.id == fixture.intent.runner.id)
            .unwrap()
            .runner
            .state,
        RunnerState::Busy
    );
    assert_eq!(*remote.calls.lock().unwrap(), vec![Call::Remove(47)]);
    assert_eq!(gateway.messages.lock().unwrap().len(), 0);
}

#[tokio::test]
async fn stale_takeout_event_uses_ordinary_reconciliation_after_reinstall() {
    let fixture = Fixture::new();
    other_organization(&fixture);
    fixture
        .store
        .begin_organization_takeout(fixture.workspace, fixture.organization)
        .unwrap();
    let event = fixture
        .store
        .pending_github_lifecycle(42)
        .unwrap()
        .remove(0)
        .event;
    let mut replacement = fixture
        .store
        .organization(fixture.workspace, fixture.organization)
        .unwrap()
        .unwrap();
    replacement.installation_id = 49;
    replacement.app_state = AppState::Active;
    fixture.store.install_organization(&replacement).unwrap();
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Busy,
            Some(47),
        )
        .unwrap();
    let remote = Remote::new(Answer::Busy);
    let gateway = Capture::new(false);
    let mut runtime = real_runtime(&fixture, remote.clone(), gateway.clone());
    let probe = Probe::present();
    runtime.installations = probe.clone();
    runtime.lifecycle(event).await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
    assert_eq!(*probe.calls.lock().unwrap(), Vec::<i64>::new());
    assert_eq!(*remote.calls.lock().unwrap(), Vec::<Call>::new());
    assert_eq!(
        gateway
            .messages
            .lock()
            .unwrap()
            .iter()
            .filter(|message| matches!(message, ControlMessage::StopRunner { .. }))
            .count(),
        0
    );
}
