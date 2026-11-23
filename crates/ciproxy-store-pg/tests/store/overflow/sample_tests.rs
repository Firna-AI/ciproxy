//! Boat sample writes use the same receipt limit across store connections.

use std::thread;

use chrono::Duration;
use ciproxy_schema::job_usage::job_usage_samples;
use ciproxy_store_interface::job_usage::{BoatSampleWrite, StoredJobUsage};
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::store::Store;
use diesel::prelude::*;
use diesel::sql_types;
use uuid::Uuid;

use crate::store::metering::concurrent::support::{backend_pid, wait_for_blocker};

use super::super::support::job_usage::sample_report;
use super::super::support::{Database, at};
use super::guard_cost_support::tagged_store;
use super::support::{create_runner_intent, save_running_delivery, setup};

#[derive(QueryableByName)]
struct Backend {
    #[diesel(sql_type = sql_types::Integer)]
    pid: i32,
}

fn running_delivery(database: &Database) -> (Uuid, StartRecord) {
    let (workspace_id, _, mut record) = setup(database);
    let hash = TokenHash::new(record.delivery_id.as_bytes().to_vec());
    create_runner_intent(database, workspace_id, &record);
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    save_running_delivery(&mut *session, &mut record, &hash);
    (workspace_id, record)
}

fn save_sample(
    store: &dyn Store,
    workspace_id: Uuid,
    record: &StartRecord,
    report: &StoredJobUsage,
) -> BoatSampleWrite {
    store
        .save_boat_sample(
            workspace_id,
            record.delivery_id,
            record.runner_id,
            report,
            report.received_at - Duration::seconds(10),
        )
        .unwrap()
}

#[test]
fn overlapping_boat_samples_save_only_the_first_report() {
    let mut database = Database::new();
    let (workspace_id, record) = running_delivery(&database);
    let store = database.store.as_ref();
    let initial = sample_report();
    assert_eq!(
        save_sample(store, workspace_id, &record, &initial),
        BoatSampleWrite::Saved
    );
    let mut first_report = sample_report();
    first_report.received_at = at() + Duration::seconds(10);
    first_report.usage.cpu_milli_vcpu = vec![50, 1_300];
    let mut second_report = sample_report();
    second_report.received_at = at() + Duration::seconds(11);
    second_report.usage.cpu_milli_vcpu = vec![75, 1_350];

    let second_pid = diesel::sql_query(
        "SELECT (SELECT pid FROM pg_stat_activity
            WHERE datname = current_database() AND pid <> pg_backend_pid()) AS pid",
    )
    .get_result::<Backend>(&mut database.connection)
    .unwrap()
    .pid;
    let first_store = tagged_store(&database.url);
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let controller_pid = backend_pid(&mut database.connection);
    let outcomes = thread::scope(|scope| {
        let (first, second) = database
            .connection
            .transaction(|controller| {
                job_usage_samples::table
                    .find((workspace_id, record.runner_id))
                    .select(job_usage_samples::runner_id)
                    .for_update()
                    .first::<Uuid>(controller)?;
                let first = scope.spawn(|| {
                    save_sample(
                        first_store.store.as_ref(),
                        workspace_id,
                        &record,
                        &first_report,
                    )
                });
                wait_for_blocker(&mut observer, first_store.pid, controller_pid);
                let second =
                    scope.spawn(|| save_sample(store, workspace_id, &record, &second_report));
                wait_for_blocker(&mut observer, second_pid, first_store.pid);
                Ok::<_, diesel::result::Error>((first, second))
            })
            .unwrap();
        (first.join().unwrap(), second.join().unwrap())
    });

    assert_eq!(outcomes, (BoatSampleWrite::Saved, BoatSampleWrite::TooSoon));
    assert_eq!(
        store.job_usage(workspace_id, record.runner_id).unwrap(),
        Some(first_report)
    );
}

#[test]
fn boat_samples_reject_each_closed_delivery_marker() {
    let database = Database::new();
    let store = database.store.as_ref();
    let closed_at = at() + Duration::seconds(20);
    for (phase, reported_at, ends_at) in [
        (StartPhase::Deleting, Some(closed_at), None),
        (StartPhase::Ended, None, Some(closed_at)),
    ] {
        let (workspace_id, mut record) = running_delivery(&database);
        let initial = sample_report();
        assert_eq!(
            save_sample(store, workspace_id, &record, &initial),
            BoatSampleWrite::Saved
        );
        record.phase = phase;
        record.reported_at = reported_at;
        record.ends_at = ends_at;
        store
            .lock_overflow(workspace_id)
            .unwrap()
            .save_start(&record)
            .unwrap();
        let mut late = sample_report();
        late.received_at = closed_at + Duration::seconds(1);

        assert_eq!(
            save_sample(store, workspace_id, &record, &late),
            BoatSampleWrite::Closed,
            "{phase:?} delivery"
        );
        assert_eq!(
            store.job_usage(workspace_id, record.runner_id).unwrap(),
            Some(initial)
        );
    }
}
