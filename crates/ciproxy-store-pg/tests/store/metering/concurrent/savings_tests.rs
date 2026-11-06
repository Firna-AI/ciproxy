//! A real report snapshot while a machine and its aggregate facts commit.

use std::thread;

use ciproxy_metering::report::range::DateRange;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_store_interface::metering::history::{MachineCost, PlanCost};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types;
use uuid::Uuid;

use crate::store::metering::support::{prices, time, usd};
use crate::store::support::{Database, machine};

use super::support::{backend_pid, tagged_store, wait_for_blocker};

#[test]
fn savings_snapshot_cannot_omit_a_concurrently_registered_machine_cost() {
    let mut database = Database::new();
    let workspace = database.legacy_workspace();
    let starts = time("2026-11-04T00:00:00Z");
    database
        .store
        .record_workspace_start(workspace.id, starts)
        .unwrap();
    database
        .store
        .save_plan_cost(
            workspace.id,
            &PlanCost {
                id: Uuid::new_v4(),
                starts_at: starts,
                ends_at: None,
                monthly_usd: usd("0"),
            },
        )
        .unwrap();
    let reader = tagged_store(&database.url);
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let controller_pid = backend_pid(&mut database.connection);
    database.connection.batch_execute("BEGIN").unwrap();
    database
        .connection
        .batch_execute("LOCK TABLE metering_daily_sizes IN ACCESS EXCLUSIVE MODE")
        .unwrap();
    let workspace_id = workspace.id;
    let range = DateRange {
        from: starts.date_naive(),
        until: time("2026-11-05T00:00:00Z").date_naive(),
    };
    let catalogs = prices();
    let read = thread::spawn(move || {
        StoredMetering::new(reader.store).savings(
            workspace_id,
            range,
            time("2026-11-04T12:00:00Z"),
            &catalogs,
        )
    });
    wait_for_blocker(&mut observer, reader.pid, controller_pid);
    let own = machine();
    database.store.create_machine(workspace.id, &own).unwrap();
    database
        .store
        .record_machine_registration(workspace.id, own.id, starts)
        .unwrap();
    database
        .store
        .save_machine_cost(
            workspace.id,
            &MachineCost {
                id: Uuid::new_v4(),
                machine_id: own.id,
                starts_at: starts,
                ends_at: None,
                monthly_usd: usd("60"),
            },
        )
        .unwrap();
    diesel::sql_query(
        "INSERT INTO metering_daily_sizes (
             workspace_id, day, vcpus, memory_bytes, disk_bytes,
             machine_starts, machine_minutes, boat_starts, boat_minutes
         ) VALUES ($1, $2, 2, 8000000000, 60000000000, 1, 2, 0, 0)",
    )
    .bind::<sql_types::Uuid, _>(workspace.id)
    .bind::<sql_types::Date, _>(range.from)
    .execute(&mut database.connection)
    .unwrap();
    database.connection.batch_execute("COMMIT").unwrap();
    let during_registration = read.join().unwrap().unwrap();
    assert_eq!(during_registration.savings[0].usd, Some(usd("0")));
    assert_eq!(during_registration.costs.missing, vec![]);
    let service: &dyn Metering = &StoredMetering::new(database.store.clone());
    let after_registration = service
        .savings(workspace.id, range, time("2026-11-04T12:00:00Z"), &prices())
        .unwrap();
    assert_eq!(
        (
            after_registration.comparisons[0].usd,
            after_registration.costs.known_machines_usd,
            after_registration.savings[0].usd
        ),
        (Some(usd("2")), usd("2"), Some(usd("0")))
    );
    assert_eq!(after_registration.costs.missing, vec![]);
}
