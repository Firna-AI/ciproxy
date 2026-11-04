//! First-removal timestamps and registered-day costs under repeated revocation.

use std::num::NonZeroU32;

use ciproxy_metering::report::costs::MonthlyDivisor;
use ciproxy_metering::report::range::DateRange;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::metering::history::{MachineCost, PlanCost};
use uuid::Uuid;

use crate::store::support::{Database, machine};

use super::support::{prices, time, usd};

#[test]
fn removal_retries_preserve_first_registration_end_and_exclude_later_cost_days() {
    let database = Database::new();
    let workspace = database.legacy_workspace();
    let starts = time("2026-11-04T00:00:00Z");
    let first = time("2026-11-04T12:00:00Z");
    let retry = time("2026-11-05T12:00:00Z");
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
    database
        .store
        .remove_machine(workspace.id, own.id, first)
        .unwrap();
    database
        .store
        .remove_machine(workspace.id, own.id, retry)
        .unwrap();
    let range = DateRange {
        from: starts.date_naive(),
        until: time("2026-11-06T00:00:00Z").date_naive(),
    };
    let service: &dyn Metering = &StoredMetering::new(database.store.clone());
    let divisor = MonthlyDivisor::FixedDays(NonZeroU32::new(30).unwrap());
    let report = service
        .savings(
            workspace.id,
            range,
            time("2026-11-06T12:00:00Z"),
            &prices(),
            divisor,
        )
        .unwrap();
    assert_eq!(report.costs.known_machines_usd, usd("2"));
    let snapshot = database
        .store
        .savings_snapshot(workspace.id, range.from, range.until)
        .unwrap();
    assert_eq!(snapshot.machines[0].removed_at, Some(first));
    let second = machine();
    database
        .store
        .create_machine(workspace.id, &second)
        .unwrap();
    database
        .store
        .record_machine_registration(workspace.id, second.id, starts)
        .unwrap();
    database
        .store
        .save_machine_cost(
            workspace.id,
            &MachineCost {
                id: Uuid::new_v4(),
                machine_id: second.id,
                starts_at: starts,
                ends_at: None,
                monthly_usd: usd("60"),
            },
        )
        .unwrap();
    database
        .store
        .revoke_workspace_execution_access(workspace.id, first)
        .unwrap();
    database
        .store
        .revoke_workspace_execution_access(workspace.id, retry)
        .unwrap();
    database
        .store
        .remove_machine(workspace.id, second.id, retry)
        .unwrap();
    let snapshot = database
        .store
        .savings_snapshot(workspace.id, range.from, range.until)
        .unwrap();
    assert_eq!(
        snapshot
            .machines
            .iter()
            .map(|machine| machine.removed_at)
            .collect::<Vec<_>>(),
        vec![Some(first), Some(first)]
    );
    let report = service
        .savings(
            workspace.id,
            range,
            time("2026-11-06T12:00:00Z"),
            &prices(),
            divisor,
        )
        .unwrap();
    assert_eq!(report.costs.known_machines_usd, usd("4"));
    let later = service
        .savings(
            workspace.id,
            DateRange {
                from: retry.date_naive(),
                until: range.until,
            },
            time("2026-11-06T12:00:00Z"),
            &prices(),
            divisor,
        )
        .unwrap();
    assert_eq!(later.costs.known_machines_usd, usd("0"));
    assert!(matches!(
        database
            .store
            .remove_machine(workspace.id, Uuid::new_v4(), retry),
        Err(Error::NotFound)
    ));
}
