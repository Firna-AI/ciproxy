//! Anonymous accounting survives organization deletion until its own retention.

use std::sync::Arc;

use chrono::Duration;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_metering::usage::PaymentNotice;
use ciproxy_store_interface::jobs::{JobResult, JobState};
use ciproxy_store_interface::metering::history::{BoatCost, Period};
use ciproxy_store_interface::runners::RunnerPlace;

use super::super::support::{Database, at, job, runner};
use super::support::{free, time, usd};

#[test]
fn anonymous_totals_survive_organization_purge_and_expire_after_ninety_days() {
    let database = Database::new();
    let workspace = database.workspace();
    let removed = database.organization(&workspace);
    let _active = database.organization(&workspace);
    let (label, scale) = database.label_and_scale_set(&removed);
    let vm = runner(
        &label,
        &scale,
        RunnerPlace::Boat {
            key_id: None,
            sandbox_id: "completed".to_owned(),
            size_name: "2cpu".to_owned(),
        },
    );
    database.store.create_runner(workspace.id, &vm).unwrap();
    let mut taken = job(&label, &scale);
    taken.runner_id = Some(vm.id);
    taken.started_at = Some(at());
    taken.completed_at = Some(at() + Duration::minutes(5));
    taken.state = JobState::Completed;
    taken.result = Some(JobResult::Success);
    database.store.insert_job(workspace.id, &taken).unwrap();
    database
        .store
        .save_boat_cost(
            workspace.id,
            &BoatCost {
                runner_id: vm.id,
                starts_at: at(),
                ends_at: Some(at() + Duration::minutes(10)),
                start_usd: usd("0.1"),
                minute_usd: usd("0.02"),
            },
        )
        .unwrap();
    let service: Arc<dyn Metering> = Arc::new(StoredMetering::new(database.store.clone()));
    let first = service
        .refresh(workspace.id, at() + Duration::hours(1))
        .unwrap();
    assert_eq!((first.updated_jobs, first.updated_sandboxes), (1, 1));
    assert_eq!(
        service
            .refresh(workspace.id, at() + Duration::hours(2))
            .unwrap()
            .updated_jobs,
        0
    );
    database
        .store
        .record_app_removal(workspace.id, removed.id, at() + Duration::hours(1))
        .unwrap();
    assert_eq!(
        database
            .store
            .purge_removed_organizations(workspace.id, time("2026-12-04T13:00:00Z"))
            .unwrap(),
        1
    );
    service
        .refresh(workspace.id, time("2026-12-04T14:00:00Z"))
        .unwrap();
    let totals = database
        .store
        .savings_snapshot(
            workspace.id,
            at().date_naive(),
            (at() + Duration::days(1)).date_naive(),
        )
        .unwrap()
        .days;
    assert_eq!(
        (
            totals[0].sizes[0].boat.starts,
            totals[0].sizes[0].boat.minutes,
            totals[0].boat_usd
        ),
        (1, usd("5"), usd("0.3"))
    );
    let current = service
        .usage(
            workspace.id,
            Period {
                starts_at: at(),
                ends_at: time("2026-12-05T12:00:00Z"),
            },
            free(),
            time("2026-12-04T14:00:00Z"),
            PaymentNotice::None,
        )
        .unwrap();
    assert_eq!((current.starts, current.organizations), (1, 1));
    let purge = service
        .refresh(workspace.id, at() + Duration::days(90))
        .unwrap();
    assert_eq!(purge.purged_rows, 3);
    assert_eq!(
        database
            .store
            .savings_snapshot(
                workspace.id,
                at().date_naive(),
                (at() + Duration::days(1)).date_naive()
            )
            .unwrap()
            .days,
        vec![]
    );
}
