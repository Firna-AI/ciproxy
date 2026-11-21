//! Full organization identities use an independent, health-checked lock namespace.

use ciproxy_store_interface::error::Error;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Integer};

use super::super::support::Database;

#[derive(QueryableByName)]
struct Lock {
    #[diesel(sql_type = BigInt)]
    first: i64,
    #[diesel(sql_type = BigInt)]
    second: i64,
    #[diesel(sql_type = Integer)]
    space: i32,
    #[diesel(sql_type = Integer)]
    pid: i32,
}

#[derive(QueryableByName)]
struct Flag {
    #[diesel(sql_type = Bool)]
    value: bool,
}

#[test]
fn organization_lease_preserves_all_64_bits_and_is_separate_from_listener_keys() {
    let mut database = Database::new();
    let github_id = i64::from_be_bytes([0xF1, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
    let mut held = database.store.lock_github_organization(github_id).unwrap();
    held.check_connection().unwrap();
    let lock = diesel::sql_query("SELECT classid::bigint AS first, objid::bigint AS second, objsubid::integer AS space, pid FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 2 AND database = (SELECT oid FROM pg_database WHERE datname = current_database())")
        .get_result::<Lock>(&mut database.connection).unwrap();
    assert_eq!(
        (lock.first, lock.second, lock.space),
        (0xF1234567, 0x89ABCDEF, 2)
    );
    let one_key = diesel::sql_query("SELECT pg_try_advisory_lock($1) AS value")
        .bind::<BigInt, _>(github_id)
        .get_result::<Flag>(&mut database.connection)
        .unwrap();
    assert!(one_key.value);
    let terminated = diesel::sql_query("SELECT pg_terminate_backend($1, 10000) AS value")
        .bind::<Integer, _>(lock.pid)
        .get_result::<Flag>(&mut database.connection)
        .unwrap();
    assert!(terminated.value);
    assert!(matches!(held.check_connection(), Err(Error::Internal(_))));
    let mut next = database.store.lock_github_organization(github_id).unwrap();
    next.check_connection().unwrap();
}
