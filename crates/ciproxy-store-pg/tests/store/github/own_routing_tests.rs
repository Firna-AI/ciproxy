//! Retired labels keep execution history while new routing uses their replacement.

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::own_runners::{OwnRunner, OwnRunnerObservation};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use diesel::prelude::*;
use diesel::sql_types;
use uuid::Uuid;

use super::super::support::{Database, at, machine, runner};
use super::fixtures::demand;

#[test]
fn retired_label_is_excluded_from_new_routing_and_reservation_without_losing_history() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (retired, old_set) = database.label_and_scale_set(&organization);
    let machine = machine();
    let store = &database.store;
    store.create_machine(workspace.id, &machine).unwrap();
    let held = OwnRunner {
        requested_size: retired.size,
        slot_indices: vec![0],
        runner: runner(
            &retired,
            &old_set,
            RunnerPlace::Machine {
                machine_id: machine.id,
                first_slot: 0,
            },
        ),
        label_name: retired.name.clone(),
        command_id: Uuid::new_v4(),
        issued_at: at(),
        acknowledged: false,
        confirmed_at: None,
    };
    store.create_own_runner(workspace.id, &held).unwrap();
    let job = demand(&retired, &old_set);
    store.save_github_job(workspace.id, &job, at()).unwrap();
    diesel::sql_query("UPDATE organization_activation SET admitted=FALSE,admin_approval_hold=TRUE WHERE organization_id=$1")
        .bind::<sql_types::Uuid,_>(organization.id).execute(&mut database.connection).unwrap();
    let mut inactive = held.clone();
    inactive.runner.id = Uuid::new_v4();
    inactive.runner.state = RunnerState::Starting;
    inactive.command_id = Uuid::new_v4();
    let outcome = store.create_own_runner(workspace.id, &inactive);
    assert!(
        matches!(outcome, Err(Error::InvalidReference)),
        "inactive reservation: {outcome:?}"
    );
    assert_eq!(store.own_runners(workspace.id).unwrap(), vec![held.clone()]);
    diesel::sql_query("UPDATE organization_activation SET admitted=TRUE,admin_approval_hold=FALSE WHERE organization_id=$1")
        .bind::<sql_types::Uuid,_>(organization.id).execute(&mut database.connection).unwrap();
    store
        .retire_dashboard_label(workspace.id, retired.id, at())
        .unwrap();
    let mut replacement = retired.clone();
    replacement.id = Uuid::new_v4();
    store.create_label(workspace.id, &replacement).unwrap();
    database.scale_set(&organization, &replacement);

    let snapshot = store.own_routing_snapshot(workspace.id).unwrap();
    assert_eq!(snapshot.labels, vec![replacement]);
    assert_eq!(snapshot.runners, vec![held.clone()]);
    assert_eq!(snapshot.jobs, vec![job]);

    let mut stale = held;
    stale.runner.id = Uuid::new_v4();
    stale.runner.state = RunnerState::Starting;
    stale.command_id = Uuid::new_v4();
    let outcome = store.create_own_runner(workspace.id, &stale);
    assert!(matches!(outcome, Err(Error::NotFound)), "{outcome:?}");
    assert_eq!(store.own_runners(workspace.id).unwrap().len(), 1);
}

#[test]
fn command_results_keep_first_confirmation_separate_from_applied_delivery() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let machine = machine();
    let store = &database.store;
    store.create_machine(workspace.id, &machine).unwrap();
    let held = OwnRunner {
        requested_size: label.size,
        slot_indices: vec![0],
        runner: runner(
            &label,
            &scale_set,
            RunnerPlace::Machine {
                machine_id: machine.id,
                first_slot: 0,
            },
        ),
        label_name: label.name,
        command_id: Uuid::new_v4(),
        issued_at: at(),
        acknowledged: false,
        confirmed_at: None,
    };
    store.create_own_runner(workspace.id, &held).unwrap();
    store
        .record_own_command_result(workspace.id, Uuid::new_v4(), true, at())
        .unwrap();
    assert_eq!(
        store.own_runners(workspace.id).unwrap()[0].confirmed_at,
        None
    );
    let confirmed = at() + chrono::Duration::seconds(2);
    store
        .record_own_command_result(workspace.id, held.command_id, false, confirmed)
        .unwrap();
    let failed = store.own_runners(workspace.id).unwrap().remove(0);
    assert_eq!(failed.confirmed_at, Some(confirmed));
    assert!(!failed.acknowledged);
    store
        .observe_own_runner(
            workspace.id,
            organization.id,
            held.runner.id,
            OwnRunnerObservation {
                state: held.runner.state,
                github_id: held.runner.github_id,
                observed_at: at() + chrono::Duration::seconds(3),
            },
        )
        .unwrap();
    let command = Uuid::new_v4();
    store
        .retire_own_runner(
            workspace.id,
            held.runner.id,
            command,
            at() + chrono::Duration::seconds(4),
            held.runner.github_id,
        )
        .unwrap();
    store
        .record_own_command_result(
            workspace.id,
            held.command_id,
            true,
            at() + chrono::Duration::seconds(5),
        )
        .unwrap();
    let stopping = store.own_runners(workspace.id).unwrap().remove(0);
    assert_eq!(stopping.confirmed_at, Some(confirmed));
    assert_eq!(stopping.command_id, command);
    assert!(!stopping.acknowledged);
    store
        .record_own_command_result(
            workspace.id,
            command,
            true,
            at() + chrono::Duration::seconds(6),
        )
        .unwrap();
    store
        .record_own_command_result(
            workspace.id,
            command,
            false,
            at() + chrono::Duration::seconds(7),
        )
        .unwrap();
    let applied = store.own_runners(workspace.id).unwrap().remove(0);
    assert!(applied.acknowledged);
    assert_eq!(applied.confirmed_at, Some(confirmed));
}
