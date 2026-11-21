//! First machine registration times through production registration writes.

use chrono::{DateTime, Duration, Utc};
use ciproxy_store_interface::metering::facts::MachineRegistration;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::Timestamptz;
use uuid::Uuid;

use crate::store::gateway_support::registration;
use crate::store::support::{Database, at, hash, machine};

#[derive(QueryableByName)]
struct CreationTime {
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

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

#[test]
fn direct_creation_keeps_its_transaction_time_after_later_update() {
    let mut database = Database::new();
    let workspace = database.workspace();
    database
        .connection
        .batch_execute(
            "CREATE TABLE machine_creation_observations (
                machine_id UUID PRIMARY KEY,
                created_at TIMESTAMPTZ NOT NULL
            );
            CREATE FUNCTION capture_machine_creation() RETURNS trigger
            LANGUAGE plpgsql AS $$
            BEGIN
                INSERT INTO machine_creation_observations (machine_id, created_at)
                VALUES (NEW.id, transaction_timestamp());
                RETURN NEW;
            END;
            $$;
            CREATE TRIGGER observe_machine_creation AFTER INSERT ON machines
            FOR EACH ROW EXECUTE FUNCTION capture_machine_creation();",
        )
        .unwrap();
    let mut own = machine();
    database.store.create_machine(workspace.id, &own).unwrap();
    let observed = diesel::sql_query(
        "SELECT created_at FROM machine_creation_observations WHERE machine_id=$1",
    )
    .bind::<diesel::sql_types::Uuid, _>(own.id)
    .get_result::<CreationTime>(&mut database.connection)
    .unwrap();
    let expected = vec![MachineRegistration {
        machine_id: own.id,
        registered_at: Some(observed.created_at),
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
    own.last_contact_at = Some(observed.created_at + Duration::days(1));
    database.store.update_machine(workspace.id, &own).unwrap();
    assert_eq!(
        database
            .store
            .workspace_usage(workspace.id)
            .unwrap()
            .machines,
        expected
    );
}
