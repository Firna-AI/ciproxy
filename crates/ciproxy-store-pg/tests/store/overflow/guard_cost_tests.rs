//! A guarded result completes while a delivery saver checks its workspace key.

use std::thread;

use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use ciproxy_store_interface::runtime_results::{OverflowRunnerResult, OverflowRunnerWrite};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types;
use rust_decimal::Decimal;

use super::super::support::Database;
use super::guard_cost_support::{backend_pid, tagged_store, wait_for_blocker};
use super::support::{create_runner_intent, reserve, setup};

const DELIVERY_GATE: i64 = 9_100_000_000;

#[derive(QueryableByName)]
struct Backend {
    #[diesel(sql_type = sql_types::Integer)]
    pid: i32,
}

#[test]
fn boat_result_and_delivery_cost_save_both_complete() {
    let mut database = Database::new();
    let (workspace, _, mut record) = setup(&database);
    create_runner_intent(&database, workspace, &record);
    record.phase = StartPhase::Running;
    record.sandbox = Some(KnownSandbox::observed(
        "guarded-cost-sandbox".to_owned(),
        record.starts_at,
        record.dispatch.as_ref().unwrap().lifetime_seconds,
    ));
    {
        let mut session = database.store.lock_overflow(workspace).unwrap();
        assert_eq!(
            reserve(
                &mut *session,
                &record,
                &EncryptedLaunch(vec![1]),
                &TokenHash::new(vec![2]),
                &ReservationBudget {
                    current_month_usd: Decimal::new(462, 3),
                    next_month_usd: None,
                },
            )
            .unwrap(),
            ReservationOutcome::Reserved
        );
    }
    let expected = database
        .store
        .overflow_runner_guard(workspace, record.runner_id)
        .unwrap()
        .unwrap();
    database
        .connection
        .batch_execute(&format!(
            "CREATE FUNCTION pause_launch_clear() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN PERFORM pg_advisory_xact_lock({DELIVERY_GATE}); RETURN NEW; END $$;
         CREATE TRIGGER pause_launch_clear BEFORE UPDATE OF launch_ciphertext
         ON overflow_starts FOR EACH ROW EXECUTE FUNCTION pause_launch_clear();"
        ))
        .unwrap();
    diesel::sql_query("SELECT pg_advisory_lock($1)")
        .bind::<sql_types::BigInt, _>(DELIVERY_GATE)
        .execute(&mut database.connection)
        .unwrap();
    let controller = backend_pid(&mut database.connection);
    let writer = tagged_store(&database.url);
    let mut session = writer.store.lock_overflow(workspace).unwrap();
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let writer_pid = diesel::sql_query("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name=(SELECT application_name FROM pg_stat_activity WHERE pid=$1) AND pid<>$1")
        .bind::<sql_types::Integer,_>(writer.pid).get_result::<Backend>(&mut observer).unwrap().pid;
    record.start_usd = Some(Decimal::new(5, 2));
    let delivery = record.delivery_id;
    let write = thread::spawn(move || session.save_start(&record));
    wait_for_blocker(&mut observer, writer_pid, controller);
    let reader = tagged_store(&database.url);
    let guard = thread::spawn(move || {
        reader.store.apply_overflow_runner_result(
            workspace,
            &expected,
            OverflowRunnerResult::Checked,
        )
    });
    wait_for_blocker(&mut observer, reader.pid, writer_pid);
    diesel::sql_query("SELECT pg_advisory_unlock($1)")
        .bind::<sql_types::BigInt, _>(DELIVERY_GATE)
        .execute(&mut database.connection)
        .unwrap();
    let saved = write.join().unwrap();
    let applied = guard.join().unwrap();
    assert!(saved.is_ok(), "delivery saver failed: {saved:?}");
    assert!(
        applied.is_ok(),
        "guard failed: {:?}",
        applied.as_ref().err()
    );
    assert!(matches!(applied.unwrap(), OverflowRunnerWrite::Applied(_)));
    let saved = database.store.overflow_runtime_snapshot(workspace).unwrap();
    let saved = saved
        .starts
        .iter()
        .find(|record| record.delivery_id == delivery)
        .unwrap();
    assert_eq!(saved.start_usd, Some(Decimal::new(5, 2)));
    assert_eq!(saved.phase, StartPhase::Running);
    assert_eq!(saved.ends_at, None);
}
