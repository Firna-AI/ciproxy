//! Current price time follows serialized writes and their lock waits.

use std::sync::{Arc, Mutex};
use std::thread;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::dashboard::records::{CurrentCostClock, CurrentMachineCost};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::store::metering::support::time;
use crate::store::support::{Database, machine};

use super::support::{backend_pid, tagged_store, wait_for_blocker};

struct Clock(Mutex<DateTime<Utc>>);
impl CurrentCostClock for Clock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap()
    }
}

fn pending(machine_id: Uuid, amount: i64) -> CurrentMachineCost {
    CurrentMachineCost {
        id: Uuid::new_v4(),
        machine_id,
        monthly_usd: Decimal::from(amount),
    }
}

#[test]
fn current_cost_uses_lock_order_and_reads_clock_after_waiting() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let machine = machine();
    database
        .store
        .create_machine(workspace.id, &machine)
        .unwrap();
    let clock = Arc::new(Clock(Mutex::new(time("2026-11-06T12:00:00Z"))));
    let delayed = pending(machine.id, 10);
    *clock.0.lock().unwrap() = time("2026-11-06T12:01:00Z");
    database
        .store
        .save_dashboard_machine_cost(workspace.id, &pending(machine.id, 20), &*clock)
        .unwrap();
    *clock.0.lock().unwrap() = time("2026-11-06T12:01:30Z");
    database
        .store
        .save_dashboard_machine_cost(workspace.id, &delayed, &*clock)
        .unwrap();
    let saved = database
        .store
        .dashboard_machine_costs(workspace.id)
        .unwrap();
    assert_eq!(saved.len(), 2);
    assert!(
        saved
            .iter()
            .all(|cost| cost.ends_at.is_none_or(|end| end >= cost.starts_at)),
        "serialized current costs must not produce inverted intervals"
    );
    assert_eq!(saved[0].ends_at, Some(saved[1].starts_at));
    assert_eq!(saved[1].monthly_usd, Decimal::from(10));

    let writer = tagged_store(&database.url);
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let controller = backend_pid(&mut database.connection);
    database.connection.batch_execute("BEGIN").unwrap();
    diesel::sql_query("SELECT id FROM workspaces WHERE id = $1 FOR UPDATE")
        .bind::<sql_types::Uuid, _>(workspace.id)
        .execute(&mut database.connection)
        .unwrap();
    let writer_clock = clock.clone();
    let operation = thread::spawn(move || {
        writer.store.save_dashboard_machine_cost(
            workspace.id,
            &pending(machine.id, 30),
            &*writer_clock,
        )
    });
    wait_for_blocker(&mut observer, writer.pid, controller);
    *clock.0.lock().unwrap() = time("2026-11-06T12:02:00Z");
    database.connection.batch_execute("COMMIT").unwrap();
    operation.join().unwrap().unwrap();
    let saved = database
        .store
        .dashboard_machine_costs(workspace.id)
        .unwrap();
    assert_eq!(saved.len(), 3);
    assert_eq!(saved[2].starts_at, time("2026-11-06T12:02:00Z"));
    assert_eq!(saved[1].ends_at, Some(time("2026-11-06T12:02:00Z")));
    assert_eq!(saved[2].ends_at, None);
    assert_eq!(saved[2].monthly_usd, Decimal::from(30));
}
