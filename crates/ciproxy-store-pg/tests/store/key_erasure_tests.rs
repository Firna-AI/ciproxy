//! Retired-key cleanup overlapping a pending live-runner registration.

use std::thread;
use std::time::{Duration, Instant};

use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types;
use url::Url;
use uuid::Uuid;

use super::support::{Database, account, at, runner};

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

/// Wait for the exact database dependency, with a deadline only for hung setup.
fn wait_for_insert_blocker(observer: &mut PgConnection, insert_pid: i32, erase_pid: i32) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = diesel::sql_query("SELECT $1 = ANY(pg_blocking_pids($2)) AS blocked")
            .bind::<sql_types::Integer, _>(insert_pid)
            .bind::<sql_types::Integer, _>(erase_pid)
            .get_result::<Blocking>(observer)
            .unwrap();
        if state.blocked {
            return;
        }
        if Instant::now() >= deadline {
            panic!("key erasure did not wait for the pending runner insert");
        }
        thread::yield_now();
    }
}

#[test]
fn retired_key_erasure_preserves_a_concurrently_registered_live_runner() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let key = account();
    let ciphertext = EncryptedBoatKey::new(vec![1, 2, 3]);
    database
        .store
        .connect_boat(workspace.id, &key, &ciphertext, 50_000_000)
        .unwrap();
    database.store.disconnect_boat(workspace.id, at()).unwrap();
    assert_eq!(
        database
            .store
            .delete_unused_boat_keys(Uuid::new_v4())
            .unwrap(),
        0
    );
    let sandbox_id = "pending-runner";
    let size_name = "small";
    let pending = runner(
        &label,
        &scale_set,
        RunnerPlace::Boat {
            key_id: Some(key.id),
            sandbox_id: sandbox_id.to_owned(),
            size_name: size_name.to_owned(),
        },
    );
    let tag = format!("key-erasure-{}", Uuid::new_v4());
    let mut url = Url::parse(&database.url).unwrap();
    url.query_pairs_mut()
        .append_pair("application_name", &tag)
        .append_pair("options", "-cstatement_timeout=20000");
    let eraser = PgStoreFactory.connect(url.as_str()).unwrap();
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let erase_pid = diesel::sql_query(
        "SELECT pid FROM pg_stat_activity
         WHERE datname = current_database() AND application_name = $1",
    )
    .bind::<sql_types::Text, _>(&tag)
    .get_result::<Backend>(&mut observer)
    .unwrap()
    .pid;
    let insert_pid = diesel::sql_query("SELECT pg_backend_pid() AS pid")
        .get_result::<Backend>(&mut database.connection)
        .unwrap()
        .pid;
    database.connection.batch_execute("BEGIN").unwrap();
    diesel::sql_query(
        "INSERT INTO runners (
             id, workspace_id, organization_id, label_id, scale_set_id, state,
             vcpus, memory_bytes, disk_bytes, boat_key_id, sandbox_id, boat_size_name, github_id
         ) VALUES ($1, $2, $3, $4, $5, 'busy', $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind::<sql_types::Uuid, _>(pending.id)
    .bind::<sql_types::Uuid, _>(workspace.id)
    .bind::<sql_types::Uuid, _>(organization.id)
    .bind::<sql_types::Uuid, _>(label.id)
    .bind::<sql_types::Uuid, _>(scale_set.id)
    .bind::<sql_types::Integer, _>(i32::try_from(pending.size.vcpu).unwrap())
    .bind::<sql_types::BigInt, _>(i64::try_from(pending.size.memory_bytes).unwrap())
    .bind::<sql_types::BigInt, _>(i64::try_from(pending.size.disk_bytes).unwrap())
    .bind::<sql_types::Uuid, _>(key.id)
    .bind::<sql_types::Text, _>(sandbox_id)
    .bind::<sql_types::Text, _>(size_name)
    .bind::<sql_types::Nullable<sql_types::BigInt>, _>(pending.github_id)
    .execute(&mut database.connection)
    .unwrap();
    let workspace_id = workspace.id;
    let erase = thread::spawn(move || eraser.delete_unused_boat_keys(workspace_id));
    wait_for_insert_blocker(&mut observer, insert_pid, erase_pid);
    database.connection.batch_execute("COMMIT").unwrap();
    let deleted = erase.join().unwrap().unwrap();
    assert_eq!(deleted, 0);
    assert_eq!(
        database
            .store
            .encrypted_boat_key(workspace.id, key.id)
            .unwrap(),
        Some(ciphertext)
    );
    assert_eq!(
        database
            .store
            .runners(workspace.id, organization.id)
            .unwrap(),
        vec![pending]
    );
}
