//! Tenant ownership through public reads, writes, and foreign references.

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use ciproxy_store_interface::settings::{OverflowWait, RepositoryAccess};
use uuid::Uuid;

use super::support::{Database, account, at, job, machine, runner};

#[test]
fn tenant_queries_isolate_reads_updates_and_related_records() {
    let database = Database::new();
    let store = &database.store;
    let owner = database.organization();
    let stranger = database.organization();
    let (mut label, scale_set) = database.label_and_scale_set(&owner);
    let mut same_name = label.clone();
    same_name.id = Uuid::new_v4();
    store.create_label(stranger.id, &same_name).unwrap();
    assert_eq!(store.labels(stranger.id).unwrap(), vec![same_name]);
    assert_eq!(store.labels(owner.id).unwrap(), vec![label.clone()]);
    label.allow_machines = false;
    label.when_nothing_fits = WhenNothingFits::LargestBoat;
    assert!(matches!(
        store.update_label(stranger.id, &label),
        Err(Error::NotFound)
    ));
    store.update_label(owner.id, &label).unwrap();
    assert_eq!(store.labels(owner.id).unwrap(), vec![label.clone()]);
    assert_eq!(store.scale_sets(owner.id).unwrap(), vec![scale_set.clone()]);
    assert_eq!(store.scale_sets(stranger.id).unwrap(), vec![]);
    let mut foreign_scale = scale_set.clone();
    foreign_scale.id = Uuid::new_v4();
    assert!(matches!(
        store.create_scale_set(stranger.id, &foreign_scale),
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

    let own_runner = runner(
        &label,
        &scale_set,
        RunnerPlace::Machine {
            machine_id: machine.id,
            first_slot: 2,
        },
    );
    assert!(matches!(
        store.create_runner(stranger.id, &own_runner),
        Err(Error::InvalidReference)
    ));
    store.create_runner(owner.id, &own_runner).unwrap();
    assert_eq!(store.runners(owner.id).unwrap(), vec![own_runner.clone()]);
    assert_eq!(store.runners(stranger.id).unwrap(), vec![]);
    assert!(matches!(
        store.update_runner(stranger.id, own_runner.id, RunnerState::Stopped, None),
        Err(Error::NotFound)
    ));
    let mut own_job = job(&label, &scale_set);
    own_job.runner_id = Some(own_runner.id);
    assert!(matches!(
        store.insert_job(stranger.id, &own_job),
        Err(Error::InvalidReference)
    ));
    assert!(store.insert_job(owner.id, &own_job).unwrap());
    assert_eq!(store.jobs(stranger.id).unwrap(), vec![]);
    assert_eq!(store.jobs(owner.id).unwrap(), vec![own_job.clone()]);
    assert!(matches!(
        store.update_job(stranger.id, &own_job),
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
    settings.repositories = RepositoryAccess::Selected(vec![7, 11]);
    store.save_settings(owner.id, &settings).unwrap();
    assert_eq!(store.settings(owner.id).unwrap(), settings);
    assert_eq!(
        store.settings(stranger.id).unwrap().repositories,
        RepositoryAccess::AllPrivate
    );
}
