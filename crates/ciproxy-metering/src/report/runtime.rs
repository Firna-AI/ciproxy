//! Elapsed fractional minutes and incremental anonymous daily corrections.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::metering::facts::{Completion, JobPlace, UsageJob};
use ciproxy_store_interface::metering::totals::{
    AccountedJob, DailyDelta, DailyTotal, LocationTotals, SizeTotal, StartTotal,
};
use rust_decimal::Decimal;

use crate::report::range::{DateRange, midnight};

/// Compute per-size totals from job facts, clipping runtime to UTC date overlap.
///
/// Starts belong to the taken-at date. A duplicate job identifier counts once.
/// Running jobs stop at the explicit observation time.
pub fn daily_totals(jobs: &[UsageJob], range: DateRange, at: DateTime<Utc>) -> Vec<DailyTotal> {
    let mut days = Vec::new();
    let mut day = range.from;
    while day < range.until {
        let Some(next) = day.succ_opt() else {
            break;
        };
        let from = midnight(day);
        let until = midnight(next).min(at);
        let mut sizes: BTreeMap<(u32, u64, u64), SizeTotal> = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for job in jobs {
            if !seen.insert(job.key()) || job.started_at > at {
                continue;
            }
            let starts = u64::from(job.started_at >= from && job.started_at < midnight(next));
            let minutes = match job.completion {
                Completion::Running => elapsed_minutes(job.started_at.max(from), at.min(until)),
                Completion::Finished(finished) => {
                    elapsed_minutes(job.started_at.max(from), finished.min(at).min(until))
                }
                Completion::TerminalUnknown => Decimal::ZERO,
            };
            if starts == 0 && minutes == Decimal::ZERO {
                continue;
            }
            let total = sizes
                .entry(size_key(job.size))
                .or_insert_with(|| SizeTotal {
                    size: job.size,
                    machines: LocationTotals::default(),
                    boat: LocationTotals::default(),
                });
            let location = match job.place {
                JobPlace::Machine => &mut total.machines,
                JobPlace::Boat => &mut total.boat,
            };
            location.starts += starts;
            location.minutes += minutes;
        }
        days.push(DailyTotal {
            day,
            sizes: sizes.into_values().collect(),
            boat_usd: Decimal::ZERO,
        });
        day = next;
    }
    days
}

/// Compute signed corrections from the previously observed job to its new facts.
///
/// The supplied range clips expired dates before any persistent write.
pub fn accounting_deltas(
    previous: Option<&AccountedJob>,
    current: &AccountedJob,
    range: DateRange,
) -> Vec<DailyDelta> {
    let mut changes: BTreeMap<_, DailyDelta> = BTreeMap::new();
    if let Some(previous) = previous {
        for total in daily_totals(
            std::slice::from_ref(&previous.job),
            range,
            previous.observed_at,
        ) {
            add_changes(&mut changes, total, -1);
        }
    }
    for total in daily_totals(
        std::slice::from_ref(&current.job),
        range,
        current.observed_at,
    ) {
        add_changes(&mut changes, total, 1);
    }
    changes
        .into_values()
        .filter(|change| {
            change.machine_starts != 0
                || change.boat_starts != 0
                || change.machine_minutes != Decimal::ZERO
                || change.boat_minutes != Decimal::ZERO
        })
        .collect()
}

/// Retract a duplicate projection only after its real numeric mapping is known.
pub fn removal_deltas(
    previous: &AccountedJob,
    range: DateRange,
) -> (Vec<DailyDelta>, Vec<StartTotal>) {
    let mut changes = BTreeMap::new();
    for total in daily_totals(
        std::slice::from_ref(&previous.job),
        range,
        previous.observed_at,
    ) {
        add_changes(&mut changes, total, -1);
    }
    let starts = if previous.job.started_at >= midnight(range.from)
        && previous.job.started_at < midnight(range.until)
        && previous.job.started_at <= previous.observed_at
    {
        vec![StartTotal {
            taken_at: previous.job.started_at,
            starts: -1,
        }]
    } else {
        Vec::new()
    };
    (changes.into_values().collect(), starts)
}

/// Correct exact taken-at counts without retaining job or organization identity.
pub fn start_deltas(
    previous: Option<&AccountedJob>,
    current: &AccountedJob,
    range: DateRange,
) -> Vec<StartTotal> {
    let mut changes: BTreeMap<_, i64> = BTreeMap::new();
    let within = |job: &AccountedJob| {
        job.job.started_at >= midnight(range.from)
            && job.job.started_at < midnight(range.until)
            && job.job.started_at <= job.observed_at
    };
    if let Some(previous) = previous.filter(|job| within(job)) {
        *changes.entry(previous.job.started_at).or_default() -= 1;
    }
    if within(current) {
        *changes.entry(current.job.started_at).or_default() += 1;
    }
    changes
        .into_iter()
        .filter(|(_, starts)| *starts != 0)
        .map(|(taken_at, starts)| StartTotal { taken_at, starts })
        .collect()
}

/// Exact elapsed fractional minutes, without invoice or display rounding.
pub fn elapsed_minutes(from: DateTime<Utc>, until: DateTime<Utc>) -> Decimal {
    if until <= from {
        return Decimal::ZERO;
    }
    let duration = until - from;
    (Decimal::from(duration.num_seconds())
        + Decimal::from(duration.subsec_nanos()) / Decimal::from(1_000_000_000))
        / Decimal::from(60)
}

fn add_changes(
    changes: &mut BTreeMap<(chrono::NaiveDate, (u32, u64, u64)), DailyDelta>,
    total: DailyTotal,
    direction: i64,
) {
    for size in total.sizes {
        let delta = changes
            .entry((total.day, size_key(size.size)))
            .or_insert(DailyDelta {
                day: total.day,
                size: size.size,
                machine_starts: 0,
                machine_minutes: Decimal::ZERO,
                boat_starts: 0,
                boat_minutes: Decimal::ZERO,
            });
        delta.machine_starts += direction * size.machines.starts as i64;
        delta.machine_minutes += Decimal::from(direction) * size.machines.minutes;
        delta.boat_starts += direction * size.boat.starts as i64;
        delta.boat_minutes += Decimal::from(direction) * size.boat.minutes;
    }
}

fn size_key(size: VmSize) -> (u32, u64, u64) {
    (size.vcpu, size.memory_bytes, size.disk_bytes)
}

#[cfg(test)]
#[path = "../_tests_/runtime_tests.rs"]
mod runtime_tests;
