//! Atomic stale-snapshot handling prevents duplicate aggregate writes.

use chrono::Duration;
use ciproxy_metering::report::range::DateRange;
use ciproxy_metering::report::runtime::{accounting_deltas, start_deltas};
use ciproxy_store_interface::jobs::JobState;
use ciproxy_store_interface::metering::totals::AccountedJob;
use ciproxy_store_interface::runners::RunnerPlace;

use super::super::support::{Database, at, job, runner};
use super::support::usd;

#[test]
fn stale_accounting_snapshot_cannot_apply_the_same_delta_twice() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale) = database.label_and_scale_set(&organization);
    let vm = runner(
        &label,
        &scale,
        RunnerPlace::Boat {
            key_id: None,
            sandbox_id: "cursor".to_owned(),
            size_name: "2cpu".to_owned(),
        },
    );
    database.store.create_runner(workspace.id, &vm).unwrap();
    let mut taken = job(&label, &scale);
    taken.runner_id = Some(vm.id);
    taken.started_at = Some(at());
    taken.state = JobState::Running;
    database.store.insert_job(workspace.id, &taken).unwrap();
    let facts = database.store.workspace_usage(workspace.id).unwrap();
    let current = AccountedJob {
        job: facts.jobs[0].clone(),
        observed_at: at() + Duration::minutes(1),
    };
    let range = DateRange {
        from: at().date_naive(),
        until: (at() + Duration::days(1)).date_naive(),
    };
    let deltas = accounting_deltas(None, &current, range);
    let starts = start_deltas(None, &current, range);
    assert!(
        database
            .store
            .apply_accounting(workspace.id, None, &current, &deltas, &starts)
            .unwrap()
    );
    assert!(
        !database
            .store
            .apply_accounting(workspace.id, None, &current, &deltas, &starts)
            .unwrap()
    );
    let totals = database
        .store
        .savings_snapshot(workspace.id, range.from, range.until)
        .unwrap()
        .days;
    assert_eq!(
        (
            totals[0].sizes[0].boat.starts,
            totals[0].sizes[0].boat.minutes
        ),
        (1, usd("1"))
    );
    database
        .store
        .purge_daily_totals(workspace.id, range.until)
        .unwrap();
    let later = AccountedJob {
        observed_at: at() + Duration::minutes(2),
        ..current.clone()
    };
    let correction = accounting_deltas(Some(&current), &later, range);
    assert!(
        database
            .store
            .apply_accounting(workspace.id, Some(&current), &later, &correction, &[])
            .unwrap()
    );
    assert_eq!(
        database
            .store
            .savings_snapshot(workspace.id, range.from, range.until)
            .unwrap()
            .days,
        vec![]
    );
}
