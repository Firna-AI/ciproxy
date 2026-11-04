//! Tagged PostgreSQL sessions and exact lock-dependency readiness signals.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::prelude::*;
use diesel::sql_types;
use url::Url;
use uuid::Uuid;

pub(super) struct TaggedStore {
    pub(super) store: Arc<dyn Store>,
    pub(super) pid: i32,
}

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

pub(super) fn tagged_store(database_url: &str) -> TaggedStore {
    let tag = format!("metering-operation-{}", Uuid::new_v4());
    let mut url = Url::parse(database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("application_name", &tag)
        .append_pair("options", "-cstatement_timeout=20000");
    let store = PgStoreFactory.connect(url.as_str()).unwrap();
    let mut observer = PgConnection::establish(database_url).unwrap();
    let pid = diesel::sql_query("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND application_name = $1")
        .bind::<sql_types::Text, _>(&tag).get_result::<Backend>(&mut observer).unwrap().pid;
    TaggedStore { store, pid }
}

pub(super) fn backend_pid(connection: &mut PgConnection) -> i32 {
    diesel::sql_query("SELECT pg_backend_pid() AS pid")
        .get_result::<Backend>(connection)
        .unwrap()
        .pid
}

/// The deadline detects hung setup, never the timing of a product outcome.
pub(super) fn wait_for_blocker(observer: &mut PgConnection, waiter: i32, blocker: i32) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = diesel::sql_query("SELECT $1 = ANY(pg_blocking_pids($2)) AS blocked")
            .bind::<sql_types::Integer, _>(blocker)
            .bind::<sql_types::Integer, _>(waiter)
            .get_result::<Blocking>(observer)
            .unwrap();
        if state.blocked {
            return;
        }
        if Instant::now() >= deadline {
            panic!("the tagged metering worker did not reach its expected blocker");
        }
        thread::yield_now();
    }
}
