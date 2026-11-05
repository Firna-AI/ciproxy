//! Monotone observation and exact cross-midnight corrections in PostgreSQL.

use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_store_interface::jobs::{JobResult, JobState};
use ciproxy_store_interface::metering::history::BoatCost;
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::store::Store;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::store::support::{Database, job, machine, runner};

use super::support::{time, usd};

fn totals(store: &dyn Store, workspace_id: Uuid) -> (Decimal, Decimal) {
    let snapshot = store
        .savings_snapshot(
            workspace_id,
            time("2026-11-04T00:00:00Z").date_naive(),
            time("2026-11-06T00:00:00Z").date_naive(),
        )
        .unwrap();
    (
        snapshot
            .days
            .iter()
            .flat_map(|day| &day.sizes)
            .map(|size| size.machines.minutes)
            .sum(),
        snapshot.days.iter().map(|day| day.boat_usd).sum(),
    )
}

#[test]
fn older_refresh_preserves_next_day_cursor_and_corrects_every_affected_date() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale) = database.label_and_scale_set(&organization);
    let own = machine();
    database.store.create_machine(workspace.id, &own).unwrap();
    let own_vm = runner(
        &label,
        &scale,
        RunnerPlace::Machine {
            machine_id: own.id,
            first_slot: 0,
        },
    );
    database.store.create_runner(workspace.id, &own_vm).unwrap();
    let boat_vm = runner(
        &label,
        &scale,
        RunnerPlace::Boat {
            key_id: None,
            sandbox_id: "midnight-cost".to_owned(),
            size_name: "small".to_owned(),
        },
    );
    database
        .store
        .create_runner(workspace.id, &boat_vm)
        .unwrap();
    let mut taken = job(&label, &scale);
    taken.runner_id = Some(own_vm.id);
    taken.started_at = Some(time("2026-11-04T23:59:00Z"));
    taken.state = JobState::Running;
    database.store.insert_job(workspace.id, &taken).unwrap();
    let open = BoatCost {
        runner_id: boat_vm.id,
        starts_at: time("2026-11-04T23:59:00Z"),
        ends_at: None,
        start_usd: usd("0"),
        minute_usd: usd("0.02"),
    };
    database.store.save_boat_cost(workspace.id, &open).unwrap();
    let service: &dyn Metering = &StoredMetering::new(database.store.clone());
    service
        .refresh(workspace.id, time("2026-11-05T00:01:00Z"))
        .unwrap();
    taken.completed_at = Some(time("2026-11-05T00:00:30Z"));
    taken.state = JobState::Completed;
    taken.result = Some(JobResult::Success);
    database.store.update_job(workspace.id, &taken).unwrap();
    database
        .store
        .save_boat_cost(
            workspace.id,
            &BoatCost {
                ends_at: Some(time("2026-11-05T00:00:30Z")),
                ..open
            },
        )
        .unwrap();
    service
        .refresh(workspace.id, time("2026-11-04T23:59:30Z"))
        .unwrap();
    assert_eq!(
        totals(&*database.store, workspace.id),
        (usd("1.5"), usd("0.03"))
    );
    let later = service
        .refresh(workspace.id, time("2026-11-05T00:02:00Z"))
        .unwrap();
    assert_eq!(
        totals(&*database.store, workspace.id),
        (usd("1.5"), usd("0.03"))
    );
    assert_eq!((later.updated_jobs, later.updated_sandboxes), (0, 0));
    let job_cursor = database
        .store
        .workspace_usage(workspace.id)
        .unwrap()
        .accounted_jobs
        .remove(0);
    assert_eq!(job_cursor.observed_at, time("2026-11-05T00:01:00Z"));
    let boat_cursor = database
        .store
        .boat_accounting(workspace.id)
        .unwrap()
        .accounted
        .remove(0);
    assert_eq!(boat_cursor.observed_at, time("2026-11-05T00:01:00Z"));
}
