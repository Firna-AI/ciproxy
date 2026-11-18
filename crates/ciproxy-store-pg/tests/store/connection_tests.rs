//! Production connection limits and recovery after bounded database waits.

use std::error;

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::sql_types::{Integer, Text};
use url::Url;

use super::support::Database;

#[derive(Debug, PartialEq, QueryableByName)]
struct Limits {
    #[diesel(sql_type = Text)]
    path: String,
    #[diesel(sql_type = Integer)]
    statement_ms: i32,
    #[diesel(sql_type = Integer)]
    lock_ms: i32,
    #[diesel(sql_type = Integer)]
    idle_transaction_ms: i32,
}

#[test]
fn shared_and_execution_connections_apply_server_limits() {
    let mut database = Database::new();
    database.connection.batch_execute(
        "CREATE TABLE observed_connection_limits (
             path text, statement_ms integer, lock_ms integer,
             idle_transaction_ms integer
         );
         CREATE FUNCTION observe_connection_limits() RETURNS trigger
         LANGUAGE plpgsql AS $$
         BEGIN
             INSERT INTO observed_connection_limits
             SELECT TG_TABLE_NAME,
                 (SELECT setting::integer FROM pg_settings WHERE name = 'statement_timeout'),
                 (SELECT setting::integer FROM pg_settings WHERE name = 'lock_timeout'),
                 (SELECT setting::integer FROM pg_settings WHERE name = 'idle_in_transaction_session_timeout');
             RETURN NEW;
         END $$;
         CREATE TRIGGER observe_limits AFTER INSERT ON workspaces
             FOR EACH ROW EXECUTE FUNCTION observe_connection_limits();
         CREATE TRIGGER observe_limits AFTER INSERT ON own_machine_leases
             FOR EACH ROW EXECUTE FUNCTION observe_connection_limits();",
    ).unwrap();
    let workspace = database.workspace();
    let mut lease = database
        .store
        .try_lock_own_runners(workspace.id)
        .unwrap()
        .unwrap();
    lease.check_connection().unwrap();
    let limits = diesel::sql_query(
        "SELECT path, statement_ms, lock_ms, idle_transaction_ms
         FROM observed_connection_limits ORDER BY path",
    )
    .load::<Limits>(&mut database.connection)
    .unwrap();
    assert_eq!(
        limits,
        vec![
            Limits {
                path: "own_machine_leases".to_owned(),
                statement_ms: 30_000,
                lock_ms: 5_000,
                idle_transaction_ms: 30_000,
            },
            Limits {
                path: "workspaces".to_owned(),
                statement_ms: 30_000,
                lock_ms: 5_000,
                idle_transaction_ms: 30_000,
            },
        ],
    );
}

#[test]
fn blocked_shared_query_uses_lock_limit_and_connection_stays_usable() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let mut url = Url::parse(&database.url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", "-cstatement_timeout=1000");
    let store = PgStoreFactory.connect(url.as_str()).unwrap();
    let failure = database
        .connection
        .transaction::<_, diesel::result::Error, _>(|connection| {
            connection.batch_execute("LOCK TABLE workspaces IN ACCESS EXCLUSIVE MODE")?;
            Ok(store.workspace(workspace.id).unwrap_err())
        })
        .unwrap();
    assert_lock_timeout(failure);
    assert_eq!(store.workspace(workspace.id).unwrap(), Some(workspace));
}

#[test]
fn waiting_organization_lease_returns_lock_timeout_without_losing_the_owner() {
    let database = Database::new();
    let workspace = database.workspace();
    let mut url = Url::parse(&database.url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", "-cstatement_timeout=1000");
    let store = PgStoreFactory.connect(url.as_str()).unwrap();
    {
        let mut held = database.store.lock_github_organization(123_456).unwrap();
        let failure = store
            .lock_github_organization(123_456)
            .err()
            .expect("competing waiting lease must reach its finite wait limit");
        assert_lock_timeout(failure);
        held.check_connection().unwrap();
        assert_eq!(store.workspace(workspace.id).unwrap(), Some(workspace));
    }
    let mut replacement = store.lock_github_organization(123_456).unwrap();
    replacement.check_connection().unwrap();
}

fn assert_lock_timeout(failure: Error) {
    let Error::Internal(internal) = failure else {
        panic!("lock timeout must preserve its database failure");
    };
    let source = error::Error::source(&internal)
        .unwrap()
        .downcast_ref::<diesel::result::Error>()
        .unwrap();
    let diesel::result::Error::DatabaseError(kind, information) = source else {
        panic!("lock timeout must be a database error");
    };
    assert_eq!(*kind, DatabaseErrorKind::Unknown);
    assert_eq!(
        information.message(),
        "canceling statement due to lock timeout"
    );
}
