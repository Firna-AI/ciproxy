//! Install-token revocation waits for registration's validated token row.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::thread;
use std::time::{Duration, Instant};

use chrono::TimeDelta;
use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types;
use url::Url;
use uuid::Uuid;

use super::gateway_support::registration;
use super::support::{Database, at, hash};

#[derive(QueryableByName)]
struct Backend {
    #[diesel(sql_type = sql_types::Integer)]
    pid: i32,
}

#[derive(QueryableByName)]
struct Blocking {
    #[diesel(sql_type = sql_types::Bool)]
    blocked: bool,
}

#[derive(Debug, Eq, PartialEq)]
enum RevocationState {
    WaitingForRegistration,
    CompletedWhileRegistrationBlocked,
}

fn blocked(observer: &mut PgConnection, blocker: i32, waiter: i32) -> bool {
    diesel::sql_query("SELECT $1 = ANY(pg_blocking_pids($2)) AS blocked")
        .bind::<sql_types::Integer, _>(blocker)
        .bind::<sql_types::Integer, _>(waiter)
        .get_result::<Blocking>(observer)
        .unwrap()
        .blocked
}

fn await_machine_insert(observer: &mut PgConnection, gate: i32, registering: i32) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if blocked(observer, gate, registering) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("registration did not reach the machine insert lock");
        }
        thread::yield_now();
    }
}

fn observe_revocation(
    observer: &mut PgConnection,
    registering: i32,
    revoking: i32,
    completed: &Receiver<()>,
) -> RevocationState {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match completed.try_recv() {
            Ok(()) => return RevocationState::CompletedWhileRegistrationBlocked,
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                panic!("revocation worker stopped without its completion signal")
            }
        }
        if blocked(observer, registering, revoking) {
            return RevocationState::WaitingForRegistration;
        }
        if Instant::now() >= deadline {
            panic!("revocation neither completed nor waited for the registration token row");
        }
        thread::yield_now();
    }
}

fn tagged_store(
    database: &Database,
    observer: &mut PgConnection,
    tag: &str,
) -> (Arc<dyn Store>, i32) {
    let mut url = Url::parse(&database.url).unwrap();
    url.query_pairs_mut()
        .append_pair("application_name", tag)
        .append_pair("options", "-cstatement_timeout=20000");
    let store = PgStoreFactory.connect(url.as_str()).unwrap();
    let pid = diesel::sql_query("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND application_name = $1")
        .bind::<sql_types::Text, _>(tag).get_result::<Backend>(observer).unwrap().pid;
    (store, pid)
}

#[test]
fn install_revocation_waits_for_validated_registration_before_it_returns() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let install_id = Uuid::new_v4();
    let install_hash = hash();
    database
        .store
        .issue_install_token(workspace.id, install_id, &install_hash, at())
        .unwrap();
    let pending = registration(workspace.id, install_hash.clone());
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let (registering, registration_pid) =
        tagged_store(&database, &mut observer, "install-registration");
    let (revoking, revocation_pid) = tagged_store(&database, &mut observer, "install-revocation");
    let gate_pid = diesel::sql_query("SELECT pg_backend_pid() AS pid")
        .get_result::<Backend>(&mut database.connection)
        .unwrap()
        .pid;
    database
        .connection
        .batch_execute("BEGIN; LOCK TABLE machines IN ACCESS EXCLUSIVE MODE")
        .unwrap();
    let workspace_id = workspace.id;
    let register = thread::spawn(move || registering.register_machine(&pending));
    await_machine_insert(&mut observer, gate_pid, registration_pid);
    let (completed, completion) = channel();
    let revoke = thread::spawn(move || {
        revoking
            .revoke_install_token(workspace_id, install_id, at() + TimeDelta::seconds(1))
            .unwrap();
        completed.send(()).unwrap();
    });
    let state = observe_revocation(&mut observer, registration_pid, revocation_pid, &completion);
    database.connection.batch_execute("COMMIT").unwrap();
    let accepted = register.join().unwrap().unwrap();
    revoke.join().unwrap();
    assert!(accepted);
    assert_eq!(state, RevocationState::WaitingForRegistration);
    assert_eq!(database.store.machines(workspace_id).unwrap().len(), 1);
    let mut later = registration(workspace_id, install_hash);
    later.contact.at = at() + TimeDelta::seconds(2);
    assert!(!database.store.register_machine(&later).unwrap());
    assert_eq!(database.store.machines(workspace_id).unwrap().len(), 1);
}
