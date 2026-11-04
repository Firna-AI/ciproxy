//! New workspace registrations never transfer retained organization history.

use chrono::Duration;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::organizations::{AppState, RepositoryAccess};
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use uuid::Uuid;

use super::support::{Database, account, at, job, machine, runner};

#[test]
fn reinstall_into_another_workspace_keeps_old_history_with_its_tenant() {
    let database = Database::new();
    let store = &database.store;
    let mut previous_workspace = database.workspace();
    let mut previous = database.organization(&previous_workspace);
    let (old_label, old_scale) = database.label_and_scale_set(&previous);
    store
        .save_repository_access(
            previous_workspace.id,
            previous.id,
            &RepositoryAccess::Selected(vec![23]),
        )
        .unwrap();
    previous.repositories = RepositoryAccess::Selected(vec![23]);
    let old_machine = machine();
    store
        .create_machine(previous_workspace.id, &old_machine)
        .unwrap();
    let mut old_account = account();
    let old_key = EncryptedBoatKey::new(vec![10, 20]);
    store
        .connect_boat(previous_workspace.id, &old_account, &old_key, 50_000_000)
        .unwrap();
    let old_runner = runner(
        &old_label,
        &old_scale,
        RunnerPlace::Boat {
            key_id: Some(old_account.id),
            sandbox_id: "retained-sandbox".to_owned(),
            size_name: "small".to_owned(),
        },
    );
    store
        .create_runner(previous_workspace.id, &old_runner)
        .unwrap();
    let mut old_job = job(&old_label, &old_scale);
    old_job.runner_id = Some(old_runner.id);
    store.insert_job(previous_workspace.id, &old_job).unwrap();
    let target_workspace = database.workspace();
    let target_organization = database.organization(&target_workspace);
    let (target_label, _) = database.label_and_scale_set(&target_organization);
    let target_machine = machine();
    store
        .create_machine(target_workspace.id, &target_machine)
        .unwrap();
    let target_account = account();
    store
        .connect_boat(
            target_workspace.id,
            &target_account,
            &EncryptedBoatKey::new(vec![30]),
            60_000_000,
        )
        .unwrap();
    let mut registration = previous.clone();
    registration.id = Uuid::new_v4();
    registration.workspace_id = target_workspace.id;
    registration.installation_id ^= 1;
    registration.repositories = RepositoryAccess::AllPrivate;
    assert!(matches!(
        store.install_organization(&registration),
        Err(Error::Conflict)
    ));
    store
        .record_app_removal(previous_workspace.id, previous.id, at())
        .unwrap();
    previous.app_state = AppState::Removed;
    previous.removed_at = Some(at());
    previous.delete_after = Some(at() + Duration::days(30));
    previous_workspace.delete_after = previous.delete_after;
    assert!(matches!(store.install_organization(&registration), Ok(())));
    assert_eq!(
        store
            .organization(previous_workspace.id, previous.id)
            .unwrap(),
        Some(previous.clone())
    );
    assert_eq!(
        store
            .organization(target_workspace.id, registration.id)
            .unwrap(),
        Some(registration.clone())
    );
    assert_eq!(
        store.jobs(previous_workspace.id, previous.id).unwrap(),
        vec![old_job]
    );
    assert_eq!(
        store.runners(previous_workspace.id, previous.id).unwrap(),
        vec![old_runner]
    );
    assert_eq!(
        store.labels(previous_workspace.id).unwrap(),
        vec![old_label]
    );
    assert_eq!(
        store.jobs(target_workspace.id, registration.id).unwrap(),
        vec![]
    );
    assert_eq!(
        store.runners(target_workspace.id, registration.id).unwrap(),
        vec![]
    );
    assert_eq!(
        store.labels(target_workspace.id).unwrap(),
        vec![target_label.clone()]
    );
    assert_eq!(
        store.machines(target_workspace.id).unwrap(),
        vec![target_machine]
    );
    old_account.retired_at = Some(at());
    assert_eq!(
        store.boat_accounts(previous_workspace.id).unwrap(),
        vec![old_account.clone()]
    );
    assert_eq!(
        store
            .encrypted_boat_key(previous_workspace.id, old_account.id)
            .unwrap(),
        Some(old_key)
    );
    assert_eq!(
        store
            .encrypted_boat_key(target_workspace.id, old_account.id)
            .unwrap(),
        None
    );
    assert_eq!(
        store.boat_accounts(target_workspace.id).unwrap(),
        vec![target_account]
    );
    let mut expected = vec![previous.clone(), registration.clone()];
    expected.sort_by_key(|record| record.id);
    assert_eq!(
        store
            .organizations_by_github_id(previous.github_id)
            .unwrap(),
        expected
    );
    assert_eq!(
        store
            .organization_by_installation_id(previous.installation_id)
            .unwrap(),
        Some(previous.clone())
    );
    assert_eq!(
        store
            .organization_by_installation_id(registration.installation_id)
            .unwrap(),
        Some(registration.clone())
    );
    assert!(matches!(
        store.install_organization(&previous),
        Err(Error::Conflict)
    ));
    assert_eq!(
        store.workspace(previous_workspace.id).unwrap(),
        Some(previous_workspace.clone())
    );
    assert!(
        store
            .purge_expired_workspace(previous_workspace.id, at() + Duration::days(30))
            .unwrap()
    );
    assert_eq!(
        store
            .organizations_by_github_id(registration.github_id)
            .unwrap(),
        vec![registration.clone()]
    );
    assert_eq!(
        store
            .organization_by_installation_id(previous.installation_id)
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .organization_by_installation_id(registration.installation_id)
            .unwrap(),
        Some(registration)
    );
    assert_eq!(
        store.workspace(target_workspace.id).unwrap(),
        Some(target_workspace.clone())
    );
    assert_eq!(
        store.labels(target_workspace.id).unwrap(),
        vec![target_label]
    );
}
