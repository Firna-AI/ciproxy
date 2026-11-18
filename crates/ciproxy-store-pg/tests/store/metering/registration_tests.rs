//! First machine registration times through production registration writes.

use chrono::Duration;
use ciproxy_store_interface::metering::facts::MachineRegistration;
use uuid::Uuid;

use crate::store::gateway_support::registration;
use crate::store::support::{Database, at, hash};

#[test]
fn gateway_registration_keeps_its_first_time_after_later_contact() {
    let database = Database::new();
    let workspace = database.workspace();
    let install = hash();
    database
        .store
        .issue_install_token(workspace.id, Uuid::new_v4(), &install, at())
        .unwrap();
    let registration = registration(workspace.id, install);
    assert!(database.store.register_machine(&registration).unwrap());
    let expected = vec![MachineRegistration {
        machine_id: registration.machine_id,
        registered_at: Some(at()),
        removed_at: None,
    }];
    assert_eq!(
        database
            .store
            .workspace_usage(workspace.id)
            .unwrap()
            .machines,
        expected
    );
    let mut later = registration.contact.clone();
    later.at += Duration::days(1);
    database
        .store
        .record_contact(workspace.id, registration.machine_id, &later)
        .unwrap();
    assert_eq!(
        database
            .store
            .workspace_usage(workspace.id)
            .unwrap()
            .machines,
        expected
    );
}
