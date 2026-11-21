//! App removal preserves only the authority needed to finish sandbox cleanup.

use chrono::Duration;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use uuid::Uuid;

use super::support::{Database, account, at, hash, machine, runner};

#[test]
fn last_app_removal_retires_keys_and_keeps_live_sandbox_reports() {
    let database = Database::new();
    let store = &database.store;
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let machine = machine();
    store.create_machine(workspace.id, &machine).unwrap();
    let install_hash = hash();
    store
        .create_install_token(workspace.id, Uuid::new_v4(), &install_hash, at())
        .unwrap();
    let machine_hash = hash();
    store
        .create_scoped_token(
            &Token {
                id: Uuid::new_v4(),
                workspace_id: workspace.id,
                subject: TokenSubject::Machine(machine.id),
                created_at: at(),
                expires_at: None,
                revoked_at: None,
            },
            &machine_hash,
        )
        .unwrap();
    let mut unused = account();
    unused.connected_at -= Duration::minutes(1);
    store
        .connect_boat(
            workspace.id,
            &unused,
            &EncryptedBoatKey::new(vec![1]),
            50_000_000,
        )
        .unwrap();
    let mut active = account();
    let ciphertext = EncryptedBoatKey::new(vec![2, 3]);
    store
        .connect_boat(workspace.id, &active, &ciphertext, 50_000_000)
        .unwrap();
    let live = runner(
        &label,
        &scale_set,
        RunnerPlace::Boat {
            key_id: Some(active.id),
            sandbox_id: "cleanup-sandbox".to_owned(),
            size_name: "small".to_owned(),
        },
    );
    store.create_runner(workspace.id, &live).unwrap();
    let sandbox_hash = hash();
    let sandbox = Token {
        id: Uuid::new_v4(),
        workspace_id: workspace.id,
        subject: TokenSubject::Sandbox {
            organization_id: organization.id,
            runner_id: live.id,
        },
        created_at: at(),
        expires_at: None,
        revoked_at: None,
    };
    store.create_scoped_token(&sandbox, &sandbox_hash).unwrap();
    let removed_at = at() + Duration::minutes(1);
    store
        .record_app_removal(workspace.id, organization.id, removed_at)
        .unwrap();
    assert_eq!(
        store.authenticate_token(&install_hash, removed_at).unwrap(),
        None
    );
    assert_eq!(
        store.authenticate_token(&machine_hash, removed_at).unwrap(),
        None
    );
    assert_eq!(
        store.authenticate_token(&sandbox_hash, removed_at).unwrap(),
        Some(sandbox.clone())
    );
    assert_eq!(store.machines(workspace.id).unwrap(), vec![]);
    active.retired_at = Some(removed_at);
    assert_eq!(
        store.boat_accounts(workspace.id).unwrap(),
        vec![active.clone()]
    );
    assert_eq!(
        store.encrypted_boat_key(workspace.id, unused.id).unwrap(),
        None
    );
    assert_eq!(
        store.encrypted_boat_key(workspace.id, active.id).unwrap(),
        Some(ciphertext.clone())
    );
    store
        .record_app_removal(
            workspace.id,
            organization.id,
            removed_at + Duration::minutes(1),
        )
        .unwrap();
    assert_eq!(
        store.boat_accounts(workspace.id).unwrap(),
        vec![active.clone()]
    );
    assert_eq!(
        store.authenticate_token(&sandbox_hash, removed_at).unwrap(),
        Some(sandbox)
    );
    store
        .update_runner(
            workspace.id,
            organization.id,
            live.id,
            RunnerState::Stopped,
            live.github_id,
        )
        .unwrap();
    assert_eq!(
        store.authenticate_token(&sandbox_hash, removed_at).unwrap(),
        None
    );
    assert_eq!(store.delete_unused_boat_keys(workspace.id).unwrap(), 1);
    assert_eq!(
        store.encrypted_boat_key(workspace.id, active.id).unwrap(),
        None
    );
}
