//! Session lock exclusion, release, and lost-session detection.

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Uuid};

use super::support::Database;

#[derive(QueryableByName)]
struct Termination {
    #[diesel(sql_type = Bool)]
    terminated: bool,
}

#[test]
fn scale_set_locks_exclude_other_sessions_and_release_on_disconnect() {
    let mut database = Database::new();
    let owner = database.workspace();
    let organization = database.organization(&owner);
    let another_organization = database.organization(&owner);
    let stranger = database.workspace();
    let (_, first) = database.label_and_scale_set(&organization);
    let (_, second) = database.label_and_scale_set(&organization);
    let other = PgStoreFactory.connect(&database.url).unwrap();
    assert!(matches!(
        other.try_lock_scale_set(stranger.id, organization.id, first.id),
        Err(Error::NotFound)
    ));
    assert!(matches!(
        other.try_lock_scale_set(owner.id, another_organization.id, first.id),
        Err(Error::NotFound)
    ));
    {
        let mut held = database
            .store
            .try_lock_scale_set(owner.id, organization.id, first.id)
            .unwrap()
            .unwrap();
        held.check_connection().unwrap();
        assert!(
            other
                .try_lock_scale_set(owner.id, organization.id, first.id)
                .unwrap()
                .is_none()
        );
        let mut independent = other
            .try_lock_scale_set(owner.id, organization.id, second.id)
            .unwrap()
            .unwrap();
        independent.check_connection().unwrap();
    }
    let mut held = other
        .try_lock_scale_set(owner.id, organization.id, first.id)
        .unwrap()
        .unwrap();
    held.check_connection().unwrap();
    let termination = diesel::sql_query(
        "SELECT pg_terminate_backend(locks.pid, 10000) AS terminated
         FROM pg_locks AS locks
         JOIN scale_sets ON locks.classid = (scale_sets.lock_key >> 32)::oid
             AND locks.objid = (scale_sets.lock_key & 4294967295)::oid
         WHERE scale_sets.id = $1 AND locks.locktype = 'advisory' AND locks.objsubid = 1
             AND locks.database = (SELECT oid FROM pg_database WHERE datname = current_database())",
    )
    .bind::<Uuid, _>(first.id)
    .get_result::<Termination>(&mut database.connection)
    .unwrap();
    assert!(termination.terminated);
    assert!(matches!(held.check_connection(), Err(Error::Internal(_))));
    let mut replacement = database
        .store
        .try_lock_scale_set(owner.id, organization.id, first.id)
        .unwrap()
        .unwrap();
    replacement.check_connection().unwrap();
}
