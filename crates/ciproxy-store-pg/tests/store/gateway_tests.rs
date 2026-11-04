//! Atomic registration and persistent connection fences against stale writers.

use std::sync::{Arc, Barrier};
use std::thread;

use chrono::Duration;
use ciproxy_protocol::cache::{DockerDiskReport, MachineCacheReport};
use ciproxy_protocol::images::ImageReport;
use ciproxy_protocol::reports::{DrainState, HardwareReport, MachineReport};
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::gateway::{Contact, Registration};
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use uuid::Uuid;

use super::support::{Database, at, hash};

fn registration(workspace_id: Uuid, install_hash: TokenHash) -> Registration {
    Registration {
        workspace_id,
        install_hash,
        machine_id: Uuid::new_v4(),
        credential_id: Uuid::new_v4(),
        credential_hash: hash(),
        contact: Contact {
            session_id: Uuid::new_v4(),
            at: at(),
            report: MachineReport {
                hostname: "builder".into(),
                agent_version: "0.1.0".into(),
                hardware: HardwareReport {
                    processor: "x64".into(),
                    threads: 4,
                    memory_bytes: 16_000_000_000,
                    kvm: false,
                    disks: vec![],
                },
                capacity: Slots(0),
                slots: vec![],
                runners: vec![],
                drain: DrainState::Active,
                cache: MachineCacheReport {
                    used_bytes: 0,
                    capacity_bytes: 0,
                    restores_last_7_days: 0,
                    hits_last_7_days: 0,
                    jobs: vec![],
                },
                docker_disks: DockerDiskReport {
                    repositories: 0,
                    used_bytes: 0,
                },
                images: ImageReport {
                    available: vec![],
                    newest: None,
                },
            },
        },
    }
}

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
