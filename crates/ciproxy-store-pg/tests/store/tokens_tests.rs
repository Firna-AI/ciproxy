//! Credential lifetime, scope, and revocation behavior.

use chrono::Duration;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use uuid::Uuid;

use super::support::{Database, at, hash, machine, runner};

#[test]
fn reusable_install_tokens_expire_and_scoped_credentials_revoke() {
    let database = Database::new();
    let store = &database.store;
    let owner = database.organization();
    let stranger = database.organization();
    let install_id = Uuid::new_v4();
    let install_hash = hash();
    store
        .create_install_token(owner.id, install_id, &install_hash, at())
        .unwrap();
    let expected = Token {
        id: install_id,
        organization_id: owner.id,
        subject: TokenSubject::Install,
        created_at: at(),
        expires_at: Some(at() + Duration::hours(1)),
        revoked_at: None,
    };
    assert_eq!(
        store.authenticate_token(&install_hash, at()).unwrap(),
        Some(expected.clone())
    );
    assert_eq!(
        store
            .authenticate_token(&install_hash, at() + Duration::minutes(59))
            .unwrap(),
        Some(expected)
    );
    assert_eq!(
        store
            .authenticate_token(&install_hash, at() + Duration::hours(1))
            .unwrap(),
        None
    );
    assert!(matches!(
        store.revoke_token(stranger.id, install_id, at()),
        Err(Error::NotFound)
    ));
    store.revoke_token(owner.id, install_id, at()).unwrap();
    assert_eq!(store.authenticate_token(&install_hash, at()).unwrap(), None);

    let machine = machine();
    store.create_machine(owner.id, &machine).unwrap();
    let credential_hash = hash();
    let credential = Token {
        id: Uuid::new_v4(),
        organization_id: owner.id,
        subject: TokenSubject::Machine(machine.id),
        created_at: at(),
        expires_at: None,
        revoked_at: None,
    };
    let mut foreign_credential = credential.clone();
    foreign_credential.organization_id = stranger.id;
    assert!(matches!(
        store.create_scoped_token(&foreign_credential, &credential_hash),
        Err(Error::InvalidReference)
    ));
    store
        .create_scoped_token(&credential, &credential_hash)
        .unwrap();
    assert_eq!(
        store
            .authenticate_token(&credential_hash, at() + Duration::days(30))
            .unwrap(),
        Some(credential.clone())
    );
    let (label, scale_set) = database.label_and_scale_set(&owner);
    let own_runner = runner(
        &label,
        &scale_set,
        RunnerPlace::Machine {
            machine_id: machine.id,
            first_slot: 0,
        },
    );
    store.create_runner(owner.id, &own_runner).unwrap();
    let sandbox_hash = hash();
    let sandbox = Token {
        id: Uuid::new_v4(),
        organization_id: owner.id,
        subject: TokenSubject::Sandbox(own_runner.id),
        created_at: at(),
        expires_at: None,
        revoked_at: None,
    };
    assert!(matches!(
        store.create_scoped_token(&sandbox, &sandbox_hash),
        Err(Error::InvalidReference)
    ));
    let boat_runner = runner(
        &label,
        &scale_set,
        RunnerPlace::Boat {
            key_id: None,
            sandbox_id: "sandbox-token".to_owned(),
            size_name: "small".to_owned(),
        },
    );
    store.create_runner(owner.id, &boat_runner).unwrap();
    let sandbox = Token {
        subject: TokenSubject::Sandbox(boat_runner.id),
        ..sandbox
    };
    store.create_scoped_token(&sandbox, &sandbox_hash).unwrap();
    assert_eq!(
        store.authenticate_token(&sandbox_hash, at()).unwrap(),
        Some(sandbox)
    );
    store.remove_machine(owner.id, machine.id, at()).unwrap();
    assert_eq!(
        store.authenticate_token(&credential_hash, at()).unwrap(),
        None
    );
    assert_eq!(store.machines(owner.id).unwrap(), vec![]);
    assert_eq!(store.authenticate_token(&hash(), at()).unwrap(), None);
}
