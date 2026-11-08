//! Runner observations retain first evidence and roll back incomplete scoped writes.

use chrono::Duration;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::own_runners::{OwnRunner, OwnRunnerObservation};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use uuid::Uuid;

use super::super::support::{Database, at, machine, runner};

#[test]
fn observation_saves_scoped_identity_state_and_first_confirmation_atomically() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let machine = machine();
    let store = &database.store;
    store.create_machine(workspace.id, &machine).unwrap();
    let mut initial = runner(
        &label,
        &scale_set,
        RunnerPlace::Machine {
            machine_id: machine.id,
            first_slot: 0,
        },
    );
    initial.state = RunnerState::Starting;
    initial.github_id = None;
    let held = OwnRunner {
        registration_scale_set_id: None,
        requested_size: label.size,
        slot_indices: vec![0],
        runner: initial,
        label_name: label.name,
        command_id: Uuid::new_v4(),
        issued_at: at(),
        acknowledged: false,
        confirmed_at: None,
    };
    store.create_own_runner(workspace.id, &held).unwrap();
    let first = at() + Duration::seconds(1);
    store
        .observe_own_runner(
            workspace.id,
            organization.id,
            held.runner.id,
            OwnRunnerObservation {
                state: RunnerState::Busy,
                github_id: Some(91),
                observed_at: first,
            },
        )
        .unwrap();
    let mut expected = held.clone();
    expected.runner.state = RunnerState::Busy;
    expected.runner.github_id = Some(91);
    expected.confirmed_at = Some(first);
    assert_eq!(
        store.own_runners(workspace.id).unwrap(),
        vec![expected.clone()]
    );
    let later = OwnRunnerObservation {
        state: RunnerState::Busy,
        github_id: Some(92),
        observed_at: at() + Duration::seconds(2),
    };
    store
        .observe_own_runner(workspace.id, organization.id, held.runner.id, later)
        .unwrap();
    expected.runner.github_id = Some(92);
    assert_eq!(
        store.own_runners(workspace.id).unwrap(),
        vec![expected.clone()]
    );

    let other_workspace = database.workspace();
    let other_organization = database.organization(&workspace);
    let invalid = OwnRunnerObservation {
        state: RunnerState::Ready,
        github_id: Some(93),
        observed_at: at() + Duration::seconds(3),
    };
    let wrong_workspace =
        store.observe_own_runner(other_workspace.id, organization.id, held.runner.id, invalid);
    assert!(
        matches!(wrong_workspace, Err(Error::NotFound)),
        "{wrong_workspace:?}"
    );
    let wrong_organization =
        store.observe_own_runner(workspace.id, other_organization.id, held.runner.id, invalid);
    assert!(
        matches!(wrong_organization, Err(Error::NotFound)),
        "{wrong_organization:?}"
    );
    assert_eq!(store.own_runners(workspace.id).unwrap(), vec![expected]);

    let mut without_command = held.runner;
    without_command.id = Uuid::new_v4();
    store.create_runner(workspace.id, &without_command).unwrap();
    let missing_command =
        store.observe_own_runner(workspace.id, organization.id, without_command.id, invalid);
    assert!(
        matches!(missing_command, Err(Error::NotFound)),
        "{missing_command:?}"
    );
    let saved = store
        .runners(workspace.id, organization.id)
        .unwrap()
        .into_iter()
        .find(|runner| runner.id == without_command.id)
        .unwrap();
    assert_eq!(saved, without_command);
}
