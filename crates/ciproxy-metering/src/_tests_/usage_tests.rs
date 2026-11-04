//! Taken-at counts, advisory limits, and whole-workspace forecasts.

use ciproxy_store_interface::metering::facts::{JobPlace, MachineRegistration, WorkspaceUsage};
use ciproxy_store_interface::metering::history::Period;
use ciproxy_store_interface::metering::totals::{AccountedJob, StartTotal};
use uuid::Uuid;

use crate::pricing::Tier;
use crate::test_support::{at, day, job, plan, usd};
use crate::usage::{Banner, PaymentNotice, workspace_usage};

fn period() -> Period {
    Period {
        starts_at: at("2026-11-04T12:00:00Z"),
        ends_at: at("2026-12-05T12:00:00Z"),
    }
}
fn jobs(count: u128) -> WorkspaceUsage {
    WorkspaceUsage {
        jobs: (1..=count)
            .map(|id| job(id, "2026-11-04T12:00:00Z", None, JobPlace::Machine))
            .collect(),
        organizations: 1,
        ..WorkspaceUsage::default()
    }
}

#[test]
fn exact_period_counts_preserve_removed_organization_starts() {
    let first = job(1, "2026-11-04T11:59:59Z", None, JobPlace::Machine);
    let mut taken = job(2, "2026-11-04T12:00:00Z", None, JobPlace::Boat);
    taken.organization_id = Uuid::from_u128(200);
    let facts = WorkspaceUsage {
        jobs: vec![
            first,
            taken.clone(),
            taken.clone(),
            job(3, "2026-12-05T12:00:00Z", None, JobPlace::Machine),
        ],
        starts: vec![
            StartTotal {
                taken_at: at("2026-11-04T13:00:00Z"),
                starts: 4,
            },
            StartTotal {
                taken_at: taken.started_at,
                starts: 1,
            },
        ],
        accounted_jobs: vec![AccountedJob {
            job: taken,
            observed_at: at("2026-11-05T00:00:00Z"),
        }],
        machines: vec![
            MachineRegistration {
                machine_id: Uuid::from_u128(1),
                registered_at: None,
                removed_at: None,
            },
            MachineRegistration {
                machine_id: Uuid::from_u128(2),
                registered_at: None,
                removed_at: Some(at("2026-11-05T00:00:00Z")),
            },
        ],
        organizations: 2,
    };
    let result = workspace_usage(
        &facts,
        period(),
        plan(Tier::Team),
        at("2026-12-05T11:00:00Z"),
        PaymentNotice::None,
    );
    assert_eq!(
        (result.starts, result.machines, result.organizations),
        (5, 1, 2)
    );
    let mut pending = facts.clone();
    let future = job(9, "2026-11-07T12:00:00Z", None, JobPlace::Machine);
    pending.jobs.push(future.clone());
    pending.accounted_jobs.push(AccountedJob {
        job: future,
        observed_at: at("2026-11-06T12:00:00Z"),
    });
    assert_eq!(
        workspace_usage(
            &pending,
            period(),
            plan(Tier::Team),
            at("2026-12-05T11:00:00Z"),
            PaymentNotice::None
        )
        .starts,
        6
    );
}

#[test]
fn forecast_includes_current_day_and_crosses_strictly_above_capacity() {
    let result = workspace_usage(
        &jobs(400),
        period(),
        plan(Tier::Free),
        at("2026-11-07T00:01:00Z"),
        PaymentNotice::None,
    );
    assert_eq!(result.expected_starts, usd("3200"));
    assert_eq!(result.crossing_day, Some(day("2026-11-24")));
    assert_eq!(result.banner, None);
    assert_eq!(result.offer.monthly_usd, usd("49"));
    let last = workspace_usage(
        &jobs(400),
        period(),
        plan(Tier::Free),
        at("2026-12-05T11:00:00Z"),
        PaymentNotice::None,
    );
    assert_eq!(last.expected_starts, usd("400"));
}

#[test]
fn soft_limits_use_strict_excess_and_payment_banner_precedence() {
    let now = at("2026-11-04T18:00:00Z");
    let exact = workspace_usage(
        &jobs(2_000),
        period(),
        plan(Tier::Free),
        now,
        PaymentNotice::None,
    );
    assert_eq!((exact.starts_over, exact.banner), (usd("0"), None));
    let mut facts = jobs(2_001);
    facts.organizations = 2;
    facts.machines = (1..=2)
        .map(|id| MachineRegistration {
            machine_id: Uuid::from_u128(id),
            registered_at: None,
            removed_at: None,
        })
        .collect();
    let over = workspace_usage(&facts, period(), plan(Tier::Free), now, PaymentNotice::None);
    assert_eq!(
        (
            over.starts_over,
            over.machines_over,
            over.organizations_over,
            over.banner
        ),
        (usd("1"), 1, 1, Some(Banner::SoftLimit))
    );
    for (notice, banner) in [
        (PaymentNotice::Failed, Banner::PaymentFailed),
        (PaymentNotice::Ended, Banner::SubscriptionEnded),
    ] {
        assert_eq!(
            workspace_usage(&facts, period(), plan(Tier::Free), now, notice).banner,
            Some(banner)
        );
    }
}
