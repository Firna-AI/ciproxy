//! Store-boundary tests of current use, atomic refresh, and retained reports.

use std::sync::{Arc, Mutex};

use ciproxy_store_interface::metering::facts::{JobPlace, WorkspaceUsage};
use ciproxy_store_interface::metering::history::{BoatAccounting, Period};
use ciproxy_store_interface::metering::report::SavingsSnapshot;
use ciproxy_store_interface::metering::store::MeteringStoreMock;
use ciproxy_store_interface::metering::totals::{
    AccountedJob, DailyTotal, LocationTotals, SizeTotal,
};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use crate::pricing::Tier;
use crate::report::catalog::Alternative;
use crate::service::{Metering, Refresh, StoredMetering};
use crate::test_support::{at, catalog, day, free_history, job, plan, range, size, usd};
use crate::usage::PaymentNotice;

#[test]
fn store_usage_applies_workspace_forecast_and_paid_offer() {
    let workspace_id = Uuid::from_u128(1);
    let facts = WorkspaceUsage {
        jobs: vec![job(1, "2026-11-04T12:00:00Z", None, JobPlace::Machine)],
        organizations: 2,
        ..WorkspaceUsage::default()
    };
    let mock = Unimock::new(
        MeteringStoreMock::workspace_usage
            .some_call(matching!(_))
            .returns(Ok(facts)),
    );
    let service: Arc<dyn Metering> = Arc::new(StoredMetering::new(Arc::new(mock)));
    let result = service
        .usage(
            workspace_id,
            Period {
                starts_at: at("2026-11-04T00:00:00Z"),
                ends_at: at("2026-12-04T00:00:00Z"),
            },
            plan(Tier::Free),
            at("2026-11-05T12:00:00Z"),
            PaymentNotice::None,
        )
        .unwrap();
    assert_eq!(
        (
            result.starts,
            result.expected_starts,
            result.organizations_over,
            result.offer.monthly_usd
        ),
        (1, usd("15"), 1, usd("49"))
    );
}

#[test]
fn refresh_applies_literal_deltas_and_reports_concurrent_writes() {
    let captured = Arc::new(Mutex::new(None));
    let saved = captured.clone();
    let mock = Unimock::new((
        MeteringStoreMock::workspace_usage
            .some_call(matching!(_))
            .returns(Ok(WorkspaceUsage {
                jobs: vec![job(
                    1,
                    "2026-11-04T12:00:00Z",
                    Some("2026-11-04T12:01:00Z"),
                    JobPlace::Machine,
                )],
                ..WorkspaceUsage::default()
            })),
        MeteringStoreMock::apply_accounting
            .some_call(matching!(_, _, _, _, _))
            .answers_arc(Arc::new(move |_, _, before, current, changes, starts| {
                *saved.lock().unwrap() = Some((
                    before.cloned(),
                    current.clone(),
                    changes.to_vec(),
                    starts.to_vec(),
                ));
                Ok(false)
            })),
        MeteringStoreMock::boat_accounting
            .some_call(matching!(_))
            .returns(Ok(BoatAccounting::default())),
        MeteringStoreMock::purge_daily_totals
            .some_call(matching!(_, _))
            .returns(Ok(3)),
    ));
    let service: Arc<dyn Metering> = Arc::new(StoredMetering::new(Arc::new(mock)));
    let result = service
        .refresh(Uuid::from_u128(1), at("2026-11-04T12:02:00Z"))
        .unwrap();
    assert_eq!(
        result,
        Refresh {
            updated_jobs: 0,
            stale_jobs: 1,
            updated_sandboxes: 0,
            stale_sandboxes: 0,
            purged_rows: 3
        }
    );
    let (before, current, changes, starts) = captured.lock().unwrap().clone().unwrap();
    assert_eq!(before, None);
    assert_eq!(current.observed_at, at("2026-11-04T12:02:00Z"));
    assert_eq!(
        (
            changes[0].machine_starts,
            changes[0].machine_minutes,
            starts[0].starts
        ),
        (1, usd("1"), 1)
    );
}

#[test]
fn refresh_skips_unchanged_completed_job_cursors() {
    let current = AccountedJob {
        job: job(
            1,
            "2026-11-04T12:00:00Z",
            Some("2026-11-04T12:01:00Z"),
            JobPlace::Boat,
        ),
        observed_at: at("2026-11-04T12:02:00Z"),
    };
    let mock = Unimock::new((
        MeteringStoreMock::workspace_usage
            .some_call(matching!(_))
            .returns(Ok(WorkspaceUsage {
                jobs: vec![current.job.clone()],
                accounted_jobs: vec![current],
                ..WorkspaceUsage::default()
            })),
        MeteringStoreMock::boat_accounting
            .some_call(matching!(_))
            .returns(Ok(BoatAccounting::default())),
        MeteringStoreMock::purge_daily_totals
            .some_call(matching!(_, _))
            .returns(Ok(0)),
    ));
    let service: Arc<dyn Metering> = Arc::new(StoredMetering::new(Arc::new(mock)));
    assert_eq!(
        service
            .refresh(Uuid::from_u128(1), at("2026-11-05T12:00:00Z"))
            .unwrap(),
        Refresh::default()
    );
}

#[test]
fn store_savings_use_retained_totals_after_job_details_disappear() {
    let retained = DailyTotal {
        day: day("2026-10-01"),
        boat_usd: usd("1"),
        sizes: vec![SizeTotal {
            size: size(),
            machines: LocationTotals {
                starts: 2,
                minutes: usd("15"),
            },
            boat: LocationTotals::default(),
        }],
    };
    let mock = Unimock::new(
        MeteringStoreMock::savings_snapshot
            .some_call(matching!(_, _, _))
            .returns(Ok(SavingsSnapshot {
                days: vec![retained],
                costs: free_history(),
                ..SavingsSnapshot::default()
            })),
    );
    let service: Arc<dyn Metering> = Arc::new(StoredMetering::new(Arc::new(mock)));
    let lists = [
        catalog(Alternative::Blacksmith, "0.2"),
        catalog(Alternative::Github, "0.4"),
    ];
    let report = service
        .savings(
            Uuid::from_u128(1),
            range("2026-10-01", "2026-11-01"),
            at("2026-11-04T00:00:00Z"),
            &lists,
        )
        .unwrap();
    assert_eq!(
        (
            report.machines.starts,
            report.machines.minutes,
            report.savings[0].usd
        ),
        (2, usd("15"), Some(usd("2")))
    );
}
