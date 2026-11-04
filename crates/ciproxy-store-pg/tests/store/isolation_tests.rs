//! Shared workspace resources and organization-specific relationships.

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::organizations::RepositoryAccess;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use ciproxy_store_interface::settings::OverflowWait;
use uuid::Uuid;

use super::support::{Database, account, at, job, machine, runner};

#[test]
fn workspace_queries_share_machines_and_keep_organization_relationships() {
    let database = Database::new();
    let store = &database.store;
    let owner = database.workspace();
    let stranger = database.workspace();
    let first = database.organization(&owner);
    let second = database.organization(&owner);
    let foreign = database.organization(&stranger);
    assert_eq!(store.workspace(owner.id).unwrap(), Some(owner.clone()));
    assert_eq!(
        store.organizations(stranger.id).unwrap(),
        vec![foreign.clone()]
    );
    assert_eq!(store.organizations(owner.id).unwrap().len(), 2);
    assert_eq!(store.organization(stranger.id, first.id).unwrap(), None);
    let mut transferred = first.clone();
    transferred.workspace_id = stranger.id;
    assert!(matches!(
        store.install_organization(&transferred),
        Err(Error::NotFound)
    ));
    let (mut label, first_scale) = database.label_and_scale_set(&first);
    let second_scale = database.scale_set(&second, &label);
    let mut foreign_label = label.clone();
    foreign_label.id = Uuid::new_v4();
    store.create_label(stranger.id, &foreign_label).unwrap();
    assert_eq!(
        store.labels(stranger.id).unwrap(),
        vec![foreign_label.clone()]
    );
    assert_eq!(store.labels(owner.id).unwrap(), vec![label.clone()]);
    label.allow_machines = false;
    label.when_nothing_fits = WhenNothingFits::LargestBoat;
    assert!(matches!(
        store.update_label(stranger.id, &label),
        Err(Error::NotFound)
    ));
    store.update_label(owner.id, &label).unwrap();
    assert_eq!(store.labels(owner.id).unwrap(), vec![label.clone()]);
    assert_eq!(
        store.scale_sets(owner.id, first.id).unwrap(),
        vec![first_scale.clone()]
    );
    assert_eq!(
        store.scale_sets(owner.id, second.id).unwrap(),
        vec![second_scale.clone()]
    );
    assert_eq!(store.scale_sets(stranger.id, first.id).unwrap(), vec![]);
    let mut wrong_label = first_scale.clone();
    wrong_label.id = Uuid::new_v4();
    wrong_label.github_id ^= 1;
    wrong_label.label_id = foreign_label.id;
    assert!(matches!(
        store.create_scale_set(owner.id, &wrong_label),
        Err(Error::InvalidReference)
    ));
    assert!(matches!(
        store.create_scale_set(stranger.id, &wrong_label),
        Err(Error::InvalidReference)
    ));

    let mut machine = machine();
    store.create_machine(owner.id, &machine).unwrap();
    machine.draining = true;
    machine.warm_runners = 0;
    machine.last_contact_at = None;
    machine.disks[0].free_bytes = 300_000_000_000;
    assert!(matches!(
        store.update_machine(stranger.id, &machine),
        Err(Error::NotFound)
    ));
    store.update_machine(owner.id, &machine).unwrap();
    assert_eq!(store.machines(owner.id).unwrap(), vec![machine.clone()]);
    assert_eq!(store.machines(stranger.id).unwrap(), vec![]);
    assert!(matches!(
        store.remove_machine(stranger.id, machine.id, at()),
        Err(Error::NotFound)
    ));
    let place = RunnerPlace::Machine {
        machine_id: machine.id,
        first_slot: 2,
    };
    let first_runner = runner(&label, &first_scale, place.clone());
    let second_runner = runner(&label, &second_scale, place);
    assert!(matches!(
        store.create_runner(stranger.id, &first_runner),
        Err(Error::InvalidReference)
    ));
    let mut wrong_organization = first_runner.clone();
    wrong_organization.organization_id = second.id;
    assert!(matches!(
        store.create_runner(owner.id, &wrong_organization),
        Err(Error::InvalidReference)
    ));
    store.create_runner(owner.id, &first_runner).unwrap();
    store.create_runner(owner.id, &second_runner).unwrap();
    assert_eq!(
        store.runners(owner.id, first.id).unwrap(),
        vec![first_runner.clone()]
    );
    assert_eq!(
        store.runners(owner.id, second.id).unwrap(),
        vec![second_runner.clone()]
    );
    assert_eq!(store.runners(stranger.id, first.id).unwrap(), vec![]);
    assert!(matches!(
        store.update_runner(
            owner.id,
            second.id,
            first_runner.id,
            RunnerState::Stopped,
            None
        ),
        Err(Error::NotFound)
    ));
    let foreign_scale = database.scale_set(&foreign, &foreign_label);
    let cross_workspace = runner(
        &foreign_label,
        &foreign_scale,
        RunnerPlace::Machine {
            machine_id: machine.id,
            first_slot: 0,
        },
    );
    assert!(matches!(
        store.create_runner(stranger.id, &cross_workspace),
        Err(Error::InvalidReference)
    ));
    let mut first_job = job(&label, &first_scale);
    first_job.runner_id = Some(first_runner.id);
    assert!(matches!(
        store.insert_job(stranger.id, &first_job),
        Err(Error::InvalidReference)
    ));
    assert!(store.insert_job(owner.id, &first_job).unwrap());
    let mut second_job = job(&label, &second_scale);
    second_job.runner_id = Some(first_runner.id);
    assert!(matches!(
        store.insert_job(owner.id, &second_job),
        Err(Error::InvalidReference)
    ));
    second_job.runner_id = Some(second_runner.id);
    assert!(store.insert_job(owner.id, &second_job).unwrap());
    assert_eq!(
        store.jobs(owner.id, first.id).unwrap(),
        vec![first_job.clone()]
    );
    assert_eq!(store.jobs(owner.id, second.id).unwrap(), vec![second_job]);
    assert_eq!(store.jobs(stranger.id, first.id).unwrap(), vec![]);
    assert!(matches!(
        store.update_job(stranger.id, &first_job),
        Err(Error::NotFound)
    ));

    let account = account();
    store
        .connect_boat(
            owner.id,
            &account,
            &EncryptedBoatKey::new(vec![9, 8, 7]),
            200_000_000,
        )
        .unwrap();
    assert_eq!(store.boat_accounts(stranger.id).unwrap(), vec![]);
    assert_eq!(
        store.encrypted_boat_key(stranger.id, account.id).unwrap(),
        None
    );
    assert!(matches!(
        store.reject_boat_key(stranger.id, account.id, at()),
        Err(Error::NotFound)
    ));
    let mut settings = store.settings(owner.id).unwrap();
    assert_eq!(settings.overflow_wait, OverflowWait::Seconds30);
    settings.overflow_enabled = false;
    settings.overflow_wait = OverflowWait::Seconds120;
    store.save_settings(owner.id, &settings).unwrap();
    assert_eq!(store.settings(owner.id).unwrap(), settings);
    let selected = RepositoryAccess::Selected(vec![7, 11]);
    store
        .save_repository_access(owner.id, first.id, &selected)
        .unwrap();
    assert!(matches!(
        store.save_repository_access(stranger.id, first.id, &selected),
        Err(Error::NotFound)
    ));
    assert_eq!(
        store
            .organization(owner.id, first.id)
            .unwrap()
            .unwrap()
            .repositories,
        selected
    );
    assert_eq!(
        store
            .organization(owner.id, second.id)
            .unwrap()
            .unwrap()
            .repositories,
        RepositoryAccess::AllPrivate
    );
}
