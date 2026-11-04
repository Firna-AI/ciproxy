//! Literal day-overlap and incremental-accounting examples.

use ciproxy_store_interface::metering::facts::JobPlace;
use ciproxy_store_interface::metering::totals::AccountedJob;

use crate::report::runtime::{accounting_deltas, daily_totals, start_deltas};
use crate::test_support::{at, day, job, range, usd};

#[test]
fn runtime_splits_at_midnight_and_starts_stay_on_taken_day() {
    let own = job(
        1,
        "2026-11-04T23:59:30Z",
        Some("2026-11-05T00:01:00Z"),
        JobPlace::Machine,
    );
    let boat = job(2, "2026-11-05T00:00:00Z", None, JobPlace::Boat);
    let days = daily_totals(
        &[own.clone(), own, boat],
        range("2026-11-04", "2026-11-06"),
        at("2026-11-05T00:02:15Z"),
    );
    assert_eq!(
        (
            days[0].sizes[0].machines.starts,
            days[0].sizes[0].machines.minutes
        ),
        (1, usd("0.5"))
    );
    assert_eq!(
        (
            days[1].sizes[0].machines.starts,
            days[1].sizes[0].machines.minutes
        ),
        (0, usd("1"))
    );
    assert_eq!(
        (days[1].sizes[0].boat.starts, days[1].sizes[0].boat.minutes),
        (1, usd("2.25"))
    );
}

#[test]
fn accounting_is_idempotent_and_corrects_late_completion() {
    let bounds = range("2026-11-04", "2026-11-05");
    let before = AccountedJob {
        job: job(1, "2026-11-04T00:00:00Z", None, JobPlace::Machine),
        observed_at: at("2026-11-04T00:01:00Z"),
    };
    let mut after = before.clone();
    after.observed_at = at("2026-11-04T00:02:00Z");
    assert_eq!(
        accounting_deltas(Some(&before), &after, bounds)[0].machine_minutes,
        usd("1")
    );
    assert_eq!(accounting_deltas(Some(&before), &before, bounds), vec![]);
    assert_eq!(start_deltas(Some(&before), &after, bounds), vec![]);
    after.job.completed_at = Some(at("2026-11-04T00:00:45Z"));
    let correction = accounting_deltas(Some(&before), &after, bounds);
    assert_eq!(
        (correction[0].machine_starts, correction[0].machine_minutes),
        (0, usd("-0.25"))
    );
    let initial = start_deltas(None, &before, bounds);
    assert_eq!(
        (initial[0].taken_at, initial[0].starts),
        (at("2026-11-04T00:00:00Z"), 1)
    );
}

#[test]
fn refresh_does_not_reinsert_expired_days_from_retained_jobs() {
    let current = AccountedJob {
        job: job(
            1,
            "2026-08-05T23:59:00Z",
            Some("2026-08-06T00:02:00Z"),
            JobPlace::Boat,
        ),
        observed_at: at("2026-11-04T12:00:00Z"),
    };
    let bounds = range("2026-08-07", "2026-11-05");
    assert_eq!(accounting_deltas(None, &current, bounds), vec![]);
    assert_eq!(start_deltas(None, &current, bounds), vec![]);
    let kept = daily_totals(
        &[current.job],
        range("2026-08-06", "2026-08-07"),
        current.observed_at,
    );
    assert_eq!(
        (kept[0].day, kept[0].sizes[0].boat.minutes),
        (day("2026-08-06"), usd("2"))
    );
}
