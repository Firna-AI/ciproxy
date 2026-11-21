//! Organization removal preserves shared resources until the last App leaves.

use chrono::Duration;
use ciproxy_store_interface::organizations::{AppState, RepositoryAccess};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use uuid::Uuid;

use super::support::installation_claim;
use super::support::{Database, account, at, hash, job, machine, runner};

#[test]
fn organization_removal_preserves_shared_access_and_its_retained_history() {
    let database = Database::new();
    let store = &database.store;
    let workspace = database.workspace();
    let mut first = database.organization(&workspace);
    let (claim_id, attempt_digest) = installation_claim::linked(store.as_ref(), &first);
    let mut second = database.organization(&workspace);
    store
        .set_app_suspended(workspace.id, second.id, true)
        .unwrap();
    second.app_state = AppState::Suspended;
    let (label, scale_set) = database.label_and_scale_set(&first);
    let second_scale = database.scale_set(&second, &label);
    let machine = machine();
    store.create_machine(workspace.id, &machine).unwrap();
    let install_hash = hash();
    let install_id = Uuid::new_v4();
    store
        .create_install_token(workspace.id, install_id, &install_hash, at())
        .unwrap();
    let install = Token {
        id: install_id,
        workspace_id: workspace.id,
        subject: TokenSubject::Install,
        created_at: at(),
        expires_at: Some(at() + Duration::hours(1)),
        revoked_at: None,
    };
    let machine_hash = hash();
    let credential = Token {
        id: Uuid::new_v4(),
        workspace_id: workspace.id,
        subject: TokenSubject::Machine(machine.id),
        created_at: at(),
        expires_at: None,
        revoked_at: None,
    };
    store
        .create_scoped_token(&credential, &machine_hash)
        .unwrap();
    let account = account();
    store
        .connect_boat(
            workspace.id,
            &account,
            &EncryptedBoatKey::new(vec![1, 2]),
            50_000_000,
        )
        .unwrap();
    let place = RunnerPlace::Boat {
        key_id: Some(account.id),
        sandbox_id: "first-sandbox".to_owned(),
        size_name: "small".to_owned(),
    };
    let first_runner = runner(&label, &scale_set, place.clone());
    let second_runner = runner(
        &label,
        &second_scale,
        RunnerPlace::Boat {
            key_id: Some(account.id),
            sandbox_id: "second-sandbox".to_owned(),
            size_name: "small".to_owned(),
        },
    );
    store.create_runner(workspace.id, &first_runner).unwrap();
    store.create_runner(workspace.id, &second_runner).unwrap();
    let sandbox_hash = hash();
    let sandbox = Token {
        id: Uuid::new_v4(),
        workspace_id: workspace.id,
        subject: TokenSubject::Sandbox {
            organization_id: first.id,
            runner_id: first_runner.id,
        },
        created_at: at(),
        expires_at: None,
        revoked_at: None,
    };
    store.create_scoped_token(&sandbox, &sandbox_hash).unwrap();
    let mut first_job = job(&label, &scale_set);
    first_job.runner_id = Some(first_runner.id);
    store.insert_job(workspace.id, &first_job).unwrap();
    let settings = store.settings(workspace.id).unwrap();
    let repositories = RepositoryAccess::Selected(vec![23, 42]);
    store
        .save_repository_access(workspace.id, first.id, &repositories)
        .unwrap();
    first.repositories = repositories;
    store
        .set_app_suspended(workspace.id, first.id, true)
        .unwrap();
    first.app_state = AppState::Suspended;
    assert_eq!(
        store.organization(workspace.id, first.id).unwrap(),
        Some(first.clone())
    );
    assert_eq!(
        store.authenticate_token(&machine_hash, at()).unwrap(),
        Some(credential.clone())
    );
    assert_eq!(
        store.boat_accounts(workspace.id).unwrap(),
        vec![account.clone()]
    );
    store
        .set_app_suspended(workspace.id, first.id, false)
        .unwrap();

    store
        .record_app_removal(workspace.id, first.id, at())
        .unwrap();
    first.app_state = AppState::Removed;
    first.removed_at = Some(at());
    first.delete_after = Some(at() + Duration::days(30));
    store
        .record_app_removal(workspace.id, first.id, at() + Duration::days(1))
        .unwrap();
    assert_eq!(
        store.organization(workspace.id, first.id).unwrap(),
        Some(first.clone())
    );
    assert_eq!(
        store.organizations_by_github_id(first.github_id).unwrap(),
        vec![first.clone()]
    );
    assert_eq!(
        store
            .organization_by_installation_id(first.installation_id)
            .unwrap(),
        Some(first.clone())
    );
    assert_eq!(
        store.authenticate_token(&install_hash, at()).unwrap(),
        Some(install.clone())
    );
    assert_eq!(
        store.authenticate_token(&machine_hash, at()).unwrap(),
        Some(credential.clone())
    );
    assert_eq!(
        store.authenticate_token(&sandbox_hash, at()).unwrap(),
        Some(sandbox)
    );
    store
        .update_runner(
            workspace.id,
            first.id,
            first_runner.id,
            RunnerState::Stopped,
            first_runner.github_id,
        )
        .unwrap();
    assert_eq!(store.authenticate_token(&sandbox_hash, at()).unwrap(), None);
    assert_eq!(
        store.workspace(workspace.id).unwrap(),
        Some(workspace.clone())
    );
    assert_eq!(
        store.boat_accounts(workspace.id).unwrap(),
        vec![account.clone()]
    );
    assert_eq!(store.machines(workspace.id).unwrap(), vec![machine.clone()]);
    assert_eq!(
        store.runners(workspace.id, second.id).unwrap(),
        vec![second_runner.clone()]
    );
    assert_eq!(
        store.jobs(workspace.id, first.id).unwrap(),
        vec![first_job.clone()]
    );
    assert_eq!(store.labels(workspace.id).unwrap(), vec![label.clone()]);
    assert_eq!(store.settings(workspace.id).unwrap(), settings);
    assert_eq!(
        store
            .purge_removed_organizations(workspace.id, at() + Duration::days(29))
            .unwrap(),
        0
    );
    store.install_organization(&first).unwrap();
    first.app_state = AppState::Active;
    first.removed_at = None;
    first.delete_after = None;
    assert_eq!(
        store.organization(workspace.id, first.id).unwrap(),
        Some(first.clone())
    );
    assert_eq!(store.jobs(workspace.id, first.id).unwrap(), vec![first_job]);
    assert_eq!(
        store.authenticate_token(&machine_hash, at()).unwrap(),
        Some(credential)
    );
    assert_eq!(
        store.boat_accounts(workspace.id).unwrap(),
        vec![account.clone()]
    );
    store
        .record_app_removal(workspace.id, first.id, at())
        .unwrap();
    assert_eq!(
        store
            .purge_removed_organizations(workspace.id, at() + Duration::days(30))
            .unwrap(),
        1
    );
    assert_eq!(store.organization(workspace.id, first.id).unwrap(), None);
    assert_eq!(
        store
            .installation_claim(claim_id)
            .unwrap()
            .map(|claim| claim.id),
        None
    );
    assert_eq!(
        store
            .installation_attempt(&attempt_digest)
            .unwrap()
            .map(|attempt| attempt.id),
        None
    );
    assert_eq!(
        store.installation_workspace_offers(claim_id, 7).unwrap(),
        Vec::<Uuid>::new()
    );
    assert_eq!(store.jobs(workspace.id, first.id).unwrap(), vec![]);
    assert_eq!(store.scale_sets(workspace.id, first.id).unwrap(), vec![]);
    assert_eq!(store.runners(workspace.id, first.id).unwrap(), vec![]);
    assert_eq!(
        store.organization(workspace.id, second.id).unwrap(),
        Some(second.clone())
    );
    assert_eq!(store.machines(workspace.id).unwrap(), vec![machine]);
    assert_eq!(store.boat_accounts(workspace.id).unwrap(), vec![account]);
}
