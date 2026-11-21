//! A committed price and cursor change between the Boat snapshot queries.

use std::thread;

use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_store_interface::metering::history::BoatCost;
use ciproxy_store_interface::metering::totals::AccountedBoat;
use ciproxy_store_interface::runners::RunnerPlace;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types;
use rust_decimal::Decimal;

use crate::store::metering::support::{time, usd};
use crate::store::support::{Database, runner};

use super::support::{backend_pid, tagged_store, wait_for_blocker};

#[test]
fn boat_read_snapshot_preserves_completed_cost_and_cursor_after_price_change() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale) = database.label_and_scale_set(&organization);
    let vm = runner(
        &label,
        &scale,
        RunnerPlace::Boat {
            key_id: None,
            sandbox_id: "price-change".to_owned(),
            size_name: "small".to_owned(),
        },
    );
    database.store.create_runner(workspace.id, &vm).unwrap();
    database
        .store
        .save_boat_cost(
            workspace.id,
            &BoatCost {
                runner_id: vm.id,
                starts_at: time("2026-11-04T12:00:00Z"),
                ends_at: None,
                start_usd: usd("0"),
                minute_usd: usd("0.02"),
            },
        )
        .unwrap();
    StoredMetering::new(database.store.clone())
        .refresh(workspace.id, time("2026-11-04T12:04:00Z"))
        .unwrap();
    let reader = tagged_store(&database.url);
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let controller_pid = backend_pid(&mut database.connection);
    database.connection.batch_execute("BEGIN").unwrap();
    database
        .connection
        .batch_execute("LOCK TABLE metering_boat_cursors IN ACCESS EXCLUSIVE MODE")
        .unwrap();
    let workspace_id = workspace.id;
    let refresh = thread::spawn(move || {
        StoredMetering::new(reader.store).refresh(workspace_id, time("2026-11-04T12:10:00Z"))
    });
    wait_for_blocker(&mut observer, reader.pid, controller_pid);
    diesel::sql_query(
        "UPDATE metering_boat_costs SET ends_at = $2, minute_usd = $3 WHERE runner_id = $1",
    )
    .bind::<sql_types::Uuid, _>(vm.id)
    .bind::<sql_types::Timestamptz, _>(time("2026-11-04T12:05:00Z"))
    .bind::<sql_types::Numeric, _>(usd("0.03"))
    .execute(&mut database.connection)
    .unwrap();
    diesel::sql_query("UPDATE metering_boat_cursors SET ends_at = $2, minute_usd = $3, observed_at = $4 WHERE runner_id = $1")
        .bind::<sql_types::Uuid, _>(vm.id)
        .bind::<sql_types::Timestamptz, _>(time("2026-11-04T12:05:00Z"))
        .bind::<sql_types::Numeric, _>(usd("0.03"))
        .bind::<sql_types::Timestamptz, _>(time("2026-11-04T12:06:00Z"))
        .execute(&mut database.connection).unwrap();
    diesel::sql_query(
        "UPDATE metering_daily_boat_costs SET usd = $2 WHERE workspace_id = $1 AND day = $3",
    )
    .bind::<sql_types::Uuid, _>(workspace.id)
    .bind::<sql_types::Numeric, _>(usd("0.15"))
    .bind::<sql_types::Date, _>(time("2026-11-04T00:00:00Z").date_naive())
    .execute(&mut database.connection)
    .unwrap();
    database.connection.batch_execute("COMMIT").unwrap();
    let outcome = refresh.join().unwrap().unwrap();
    let snapshot = database
        .store
        .savings_snapshot(
            workspace.id,
            time("2026-11-04T00:00:00Z").date_naive(),
            time("2026-11-05T00:00:00Z").date_naive(),
        )
        .unwrap();
    assert_eq!(
        snapshot
            .days
            .iter()
            .map(|day| day.boat_usd)
            .sum::<Decimal>(),
        usd("0.15")
    );
    assert_eq!((outcome.updated_sandboxes, outcome.stale_sandboxes), (0, 1));
    let cursor = database
        .store
        .boat_accounting(workspace.id)
        .unwrap()
        .accounted
        .remove(0);
    assert_eq!(
        cursor,
        AccountedBoat {
            cost: BoatCost {
                runner_id: vm.id,
                starts_at: time("2026-11-04T12:00:00Z"),
                ends_at: Some(time("2026-11-04T12:05:00Z")),
                start_usd: usd("0"),
                minute_usd: usd("0.03")
            },
            observed_at: time("2026-11-04T12:06:00Z")
        }
    );
}
