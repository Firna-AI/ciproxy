//! A real stale Boat refresh behind a competing cursor and aggregate transaction.

use std::thread;

use ciproxy_metering::report::boat::boat_cost_deltas;
use ciproxy_metering::report::range::DateRange;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_schema::runners::runners;
use ciproxy_store_interface::metering::history::BoatCost;
use ciproxy_store_interface::metering::totals::AccountedBoat;
use ciproxy_store_interface::runners::RunnerPlace;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::store::metering::support::{time, usd};
use crate::store::support::{Database, runner};

use super::support::{backend_pid, tagged_store, wait_for_blocker};

#[test]
fn coherent_boat_snapshot_cannot_restore_a_competing_completed_cost() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale) = database.label_and_scale_set(&organization);
    let vm = runner(
        &label,
        &scale,
        RunnerPlace::Boat {
            key_id: None,
            sandbox_id: "competing-cost".to_owned(),
            size_name: "small".to_owned(),
        },
    );
    database.store.create_runner(workspace.id, &vm).unwrap();
    let open = BoatCost {
        runner_id: vm.id,
        starts_at: time("2026-11-04T12:00:00Z"),
        ends_at: None,
        start_usd: usd("0"),
        minute_usd: usd("0.02"),
    };
    database.store.save_boat_cost(workspace.id, &open).unwrap();
    StoredMetering::new(database.store.clone())
        .refresh(workspace.id, time("2026-11-04T12:05:00Z"))
        .unwrap();
    let old = database
        .store
        .boat_accounting(workspace.id)
        .unwrap()
        .accounted
        .remove(0);
    let completed = AccountedBoat {
        cost: BoatCost {
            ends_at: Some(time("2026-11-04T12:05:00Z")),
            ..open
        },
        observed_at: time("2026-11-04T12:06:00Z"),
    };
    let range = DateRange {
        from: time("2026-11-04T00:00:00Z").date_naive(),
        until: time("2026-11-05T00:00:00Z").date_naive(),
    };
    let deltas = boat_cost_deltas(Some(&old), &completed, range);
    let writer = tagged_store(&database.url);
    let reader = tagged_store(&database.url);
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let controller_pid = backend_pid(&mut database.connection);
    database.connection.batch_execute("BEGIN").unwrap();
    runners::table
        .find(vm.id)
        .select(runners::id)
        .for_update()
        .first::<Uuid>(&mut database.connection)
        .unwrap();
    diesel::sql_query("UPDATE metering_boat_costs SET ends_at = $2 WHERE runner_id = $1")
        .bind::<sql_types::Uuid, _>(vm.id)
        .bind::<sql_types::Timestamptz, _>(time("2026-11-04T12:05:00Z"))
        .execute(&mut database.connection)
        .unwrap();
    let workspace_id = workspace.id;
    let write = thread::spawn(move || {
        writer
            .store
            .apply_boat_accounting(workspace_id, Some(&old), &completed, &deltas)
    });
    wait_for_blocker(&mut observer, writer.pid, controller_pid);
    let refresh = thread::spawn(move || {
        StoredMetering::new(reader.store).refresh(workspace_id, time("2026-11-04T12:10:00Z"))
    });
    wait_for_blocker(&mut observer, reader.pid, writer.pid);
    database.connection.batch_execute("COMMIT").unwrap();
    assert!(write.join().unwrap().unwrap());
    let outcome = refresh.join().unwrap().unwrap();
    let snapshot = database
        .store
        .savings_snapshot(workspace.id, range.from, range.until)
        .unwrap();
    assert_eq!(
        snapshot
            .days
            .iter()
            .map(|day| day.boat_usd)
            .sum::<Decimal>(),
        usd("0.10")
    );
    assert_eq!((outcome.updated_sandboxes, outcome.stale_sandboxes), (0, 1));
    let cursor = database
        .store
        .boat_accounting(workspace.id)
        .unwrap()
        .accounted
        .remove(0);
    assert_eq!(cursor.cost.ends_at, Some(time("2026-11-04T12:05:00Z")));
}
