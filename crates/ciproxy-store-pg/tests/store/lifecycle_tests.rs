//! Suspension, App removal, reinstall, and retained-data expiry.

use chrono::Duration;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use uuid::Uuid;

use super::support::{Database, account, at, hash, job, machine, runner};

#[test]
fn app_removal_revokes_credentials_and_keeps_data_for_thirty_days() {
    let database = Database::new();
    let store = &database.store;
    let mut owner = database.organization();
    let (label, scale_set) = database.label_and_scale_set(&owner);
    let machine = machine();
    store.create_machine(owner.id, &machine).unwrap();
    let install_hash = hash();
    store
        .create_install_token(owner.id, Uuid::new_v4(), &install_hash, at())
        .unwrap();
    let machine_hash = hash();
    store
        .create_scoped_token(
            &Token {
                id: Uuid::new_v4(),
                organization_id: owner.id,
                subject: TokenSubject::Machine(machine.id),
                created_at: at(),
                expires_at: None,
                revoked_at: None,
            },
            &machine_hash,
        )
        .unwrap();
    let account = account();
    store
        .connect_boat(
            owner.id,
            &account,
            &EncryptedBoatKey::new(vec![1, 2]),
            50_000_000,
        )
        .unwrap();
    let mut live = runner(
        &label,
        &scale_set,
        RunnerPlace::Boat {
            key_id: Some(account.id),
            sandbox_id: "running".to_owned(),
            size_name: "small".to_owned(),
        },
    );
    store.create_runner(owner.id, &live).unwrap();
    let sandbox_hash = hash();
    store
        .create_scoped_token(
            &Token {
                id: Uuid::new_v4(),
                organization_id: owner.id,
                subject: TokenSubject::Sandbox(live.id),
                created_at: at(),
                expires_at: None,
                revoked_at: None,
            },
            &sandbox_hash,
        )
        .unwrap();
    let mut job = job(&label, &scale_set);
    job.runner_id = Some(live.id);
    store.insert_job(owner.id, &job).unwrap();
    let settings = store.settings(owner.id).unwrap();
    store.set_app_suspended(owner.id, true).unwrap();
    owner.app_state = AppState::Suspended;
    assert_eq!(store.organization(owner.id).unwrap(), Some(owner.clone()));
    assert_eq!(
        store.organization_by_github_id(owner.github_id).unwrap(),
        Some(owner.clone())
    );
    assert_eq!(
        store
            .organization_by_installation_id(owner.installation_id)
            .unwrap(),
        Some(owner.clone())
    );
    assert_eq!(
        store
            .authenticate_token(&machine_hash, at())
            .unwrap()
            .unwrap()
            .organization_id,
        owner.id
    );
    assert_eq!(
        store.boat_accounts(owner.id).unwrap(),
        vec![account.clone()]
    );
    store.set_app_suspended(owner.id, false).unwrap();

    store.record_app_removal(owner.id, at()).unwrap();
    owner.app_state = AppState::Removed;
    owner.removed_at = Some(at());
    owner.delete_after = Some(at() + Duration::days(30));
    assert_eq!(store.organization(owner.id).unwrap(), Some(owner.clone()));
    assert_eq!(store.authenticate_token(&install_hash, at()).unwrap(), None);
    assert_eq!(store.authenticate_token(&machine_hash, at()).unwrap(), None);
    assert_eq!(store.authenticate_token(&sandbox_hash, at()).unwrap(), None);
    assert_eq!(store.boat_accounts(owner.id).unwrap(), vec![]);
    assert_eq!(store.machines(owner.id).unwrap(), vec![]);
    live.place = RunnerPlace::Boat {
        key_id: None,
        sandbox_id: "running".to_owned(),
        size_name: "small".to_owned(),
    };
    assert_eq!(store.runners(owner.id).unwrap(), vec![live]);
    assert_eq!(store.jobs(owner.id).unwrap(), vec![job.clone()]);
    assert_eq!(store.labels(owner.id).unwrap(), vec![label.clone()]);
    assert_eq!(store.settings(owner.id).unwrap(), settings);
    assert_eq!(
        store
            .purge_removed_organizations(at() + Duration::days(29))
            .unwrap(),
        0
    );
    store.install_organization(&owner).unwrap();
    owner.app_state = AppState::Active;
    owner.removed_at = None;
    owner.delete_after = None;
    assert_eq!(store.organization(owner.id).unwrap(), Some(owner.clone()));
    assert_eq!(store.jobs(owner.id).unwrap(), vec![job]);
    assert_eq!(store.authenticate_token(&machine_hash, at()).unwrap(), None);
    assert_eq!(store.settings(owner.id).unwrap(), settings);
    assert_eq!(store.boat_accounts(owner.id).unwrap(), vec![]);
    store.record_app_removal(owner.id, at()).unwrap();
    assert_eq!(
        store
            .purge_removed_organizations(at() + Duration::days(30))
            .unwrap(),
        1
    );
    assert_eq!(store.organization(owner.id).unwrap(), None);
    assert_eq!(store.labels(owner.id).unwrap(), vec![]);
    assert_eq!(store.jobs(owner.id).unwrap(), vec![]);
}
