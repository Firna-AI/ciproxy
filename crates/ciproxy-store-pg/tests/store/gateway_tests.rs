//! Atomic registration and persistent connection fences against stale writers.

use std::sync::{Arc, Barrier};
use std::thread;

use chrono::Duration;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::gateway::Contact;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use uuid::Uuid;

use super::gateway_support::registration;
use super::support::{Database, at, hash};

#[test]
fn rolls_back_registration_and_fences_replaced_or_removed_sessions() {
    let database = Database::new();
    let workspace = database.workspace();
    let token_hash = hash();
    database
        .store
        .issue_install_token(workspace.id, Uuid::new_v4(), &token_hash, at())
        .unwrap();
    let mut registration = registration(workspace.id, token_hash.clone());
    registration.credential_hash = token_hash;
    assert!(matches!(
        database.store.register_machine(&registration),
        Err(Error::Conflict)
    ));
    assert_eq!(database.store.machines(workspace.id).unwrap(), vec![]);
    registration.credential_hash = hash();
    assert!(database.store.register_machine(&registration).unwrap());
    let mut settings = database.store.machines(workspace.id).unwrap().remove(0);
    settings.warm_runners = 7;
    database
        .store
        .update_machine(workspace.id, &settings)
        .unwrap();
    database
        .store
        .set_machine_draining(workspace.id, registration.machine_id, true)
        .unwrap();
    let newer = Contact {
        session_id: Uuid::new_v4(),
        ..registration.contact.clone()
    };
    let machine = database
        .store
        .connect_machine(
            workspace.id,
            registration.machine_id,
            &registration.credential_hash,
            &newer,
        )
        .unwrap()
        .unwrap();
    assert!(machine.draining);
    assert_eq!(machine.warm_runners, 7);
    database
        .store
        .record_contact(workspace.id, registration.machine_id, &newer)
        .unwrap();
    database
        .store
        .record_activity(
            workspace.id,
            registration.machine_id,
            newer.session_id,
            at() + Duration::seconds(15),
        )
        .unwrap();
    assert!(matches!(
        database.store.record_activity(
            workspace.id,
            registration.machine_id,
            registration.contact.session_id,
            at() + Duration::seconds(30)
        ),
        Err(Error::Conflict)
    ));
    let observed = database
        .store
        .machine_snapshots(workspace.id)
        .unwrap()
        .remove(0);
    assert_eq!(
        observed.machine.last_contact_at,
        Some(at() + Duration::seconds(15))
    );
    assert_eq!(observed.report, Some(newer.report.clone()));
    assert!(observed.machine.draining);
    assert_eq!(observed.machine.warm_runners, 7);
    assert!(matches!(
        database
            .store
            .record_contact(workspace.id, registration.machine_id, &registration.contact),
        Err(Error::Conflict)
    ));
    database
        .store
        .revoke_machine(workspace.id, registration.machine_id, at())
        .unwrap();
    assert!(matches!(
        database
            .store
            .record_contact(workspace.id, registration.machine_id, &newer),
        Err(Error::NotFound)
    ));
    assert_eq!(
        database
            .store
            .authenticate_token(&registration.credential_hash, at())
            .unwrap(),
        None
    );
    assert_eq!(
        database.store.machine_snapshots(workspace.id).unwrap(),
        vec![]
    );
}

#[test]
fn serializes_registration_and_issuance_against_last_app_removal() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let token_hash = hash();
    database
        .store
        .issue_install_token(workspace.id, Uuid::new_v4(), &token_hash, at())
        .unwrap();
    let registration = registration(workspace.id, token_hash);
    let candidate = registration.clone();
    let concurrent = PgStoreFactory.connect(&database.url).unwrap();
    let removal = database.store.clone();
    let barrier = Arc::new(Barrier::new(3));
    let registering = barrier.clone();
    let removing = barrier.clone();
    let register = thread::spawn(move || {
        registering.wait();
        concurrent.register_machine(&candidate).unwrap()
    });
    let remove = thread::spawn(move || {
        removing.wait();
        removal
            .record_app_removal(workspace.id, organization.id, at())
            .unwrap();
    });
    barrier.wait();
    let _accepted_before_removal = register.join().unwrap();
    remove.join().unwrap();
    assert_eq!(
        database.store.machine_snapshots(workspace.id).unwrap(),
        vec![]
    );
    assert_eq!(
        database
            .store
            .authenticate_token(&registration.credential_hash, at())
            .unwrap(),
        None
    );
    assert!(!database.store.register_machine(&registration).unwrap());
    assert!(matches!(
        database
            .store
            .issue_install_token(workspace.id, Uuid::new_v4(), &hash(), at()),
        Err(Error::NotFound)
    ));
}
