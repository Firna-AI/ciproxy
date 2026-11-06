//! Retired labels keep execution history while new routing uses their replacement.

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use uuid::Uuid;

use super::super::support::{Database, at, machine, runner};
use super::fixtures::demand;

#[test]
fn retired_label_is_excluded_from_new_routing_and_reservation_without_losing_history() {
    let database = Database::new();
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
    };
    store.create_own_runner(workspace.id, &held).unwrap();
    let job = demand(&retired, &old_set);
    store.save_github_job(workspace.id, &job, at()).unwrap();
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
