//! Durable facts and stable remote-resource repair preserve tenant authority.

use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::runners::RunnerPlace;
use uuid::Uuid;

use super::super::support::{Database, at, job, machine, runner};
use super::fixtures::demand;

#[test]
fn saved_run_facts_are_scoped_to_workspace_and_organization() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let other = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let (other_label, other_set) = database.label_and_scale_set(&other);
    let original = demand(&label, &scale_set);
    let mut different = demand(&other_label, &other_set);
    different.facts.head_sha = "different-trusted-head".to_owned();
    database
        .store
        .save_github_job(workspace.id, &original, at())
        .unwrap();
    database
        .store
        .save_github_job(workspace.id, &different, at())
        .unwrap();
    assert_eq!(
        database
            .store
            .github_run_facts(workspace.id, organization.id, 31)
            .unwrap(),
        Some(original.facts)
    );
    assert_eq!(
        database
            .store
            .github_run_facts(workspace.id, other.id, 31)
            .unwrap(),
        Some(different.facts)
    );
    assert_eq!(
        database
            .store
            .github_run_facts(Uuid::new_v4(), organization.id, 31)
            .unwrap(),
        None
    );
    assert_eq!(
        database
            .store
            .github_run_facts(workspace.id, Uuid::new_v4(), 31)
            .unwrap(),
        None
    );
    assert_eq!(
        database
            .store
            .github_run_facts(workspace.id, organization.id, 32)
            .unwrap(),
        None
    );
}

#[test]
fn remote_scale_set_repair_preserves_internal_lock_and_history() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let store = &database.store;
    let job = job(&label, &scale_set);
    let demand = demand(&label, &scale_set);
    let machine = machine();
    store.create_machine(workspace.id, &machine).unwrap();
    let runner = runner(
        &label,
        &scale_set,
        RunnerPlace::Machine {
            machine_id: machine.id,
            first_slot: 0,
        },
    );
    store.create_runner(workspace.id, &runner).unwrap();
    store.insert_job(workspace.id, &job).unwrap();
    store.save_github_job(workspace.id, &demand, at()).unwrap();
    let _lease = store
        .try_lock_scale_set(workspace.id, organization.id, scale_set.id)
        .unwrap()
        .unwrap();
    let mut requested = scale_set.clone();
    requested.id = Uuid::new_v4();
    requested.github_id = 83;
    requested.runner_group_id = 97;
    let saved = store
        .save_github_scale_set(workspace.id, &requested)
        .unwrap();
    let mut expected = scale_set.clone();
    expected.github_id = 83;
    expected.runner_group_id = 97;
    assert_eq!(saved, expected);
    assert!(
        store
            .try_lock_scale_set(workspace.id, organization.id, scale_set.id)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store.jobs(workspace.id, organization.id).unwrap(),
        vec![job]
    );
    assert_eq!(
        store.github_jobs(workspace.id, organization.id).unwrap(),
        vec![demand]
    );
    assert_eq!(
        store.runners(workspace.id, organization.id).unwrap(),
        vec![runner]
    );
    assert_eq!(
        store.github_runner_group(organization.github_id).unwrap(),
        Some(97)
    );
    store
        .record_app_removal(workspace.id, organization.id, at())
        .unwrap();
    let next_workspace = database.workspace();
    let next = Organization {
        id: Uuid::new_v4(),
        workspace_id: next_workspace.id,
        installation_id: organization.installation_id ^ 1,
        ..organization.clone()
    };
    store.install_organization(&next).unwrap();
    assert_eq!(
        store.github_runner_group(organization.github_id).unwrap(),
        Some(97)
    );
    assert!(matches!(
        store.save_github_scale_set(next_workspace.id, &requested),
        Err(ciproxy_store_interface::error::Error::NotFound)
    ));
    assert_eq!(
        store.scale_sets(workspace.id, organization.id).unwrap(),
        vec![expected]
    );
}
