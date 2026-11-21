//! Identity-free daily Boat costs and incremental cost corrections.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::metering::history::BoatCost;
use ciproxy_store_interface::metering::totals::{AccountedBoat, DailyBoatCost};
use rust_decimal::Decimal;

use crate::report::range::{DateRange, midnight};
use crate::report::runtime::elapsed_minutes;

/// Allocate start charges to their start date and runtime to UTC day overlap.
pub fn daily_boat_costs(
    costs: &[BoatCost],
    range: DateRange,
    at: DateTime<Utc>,
) -> Vec<DailyBoatCost> {
    let mut days = Vec::new();
    let mut day = range.from;
    while day < range.until && day <= at.date_naive() {
        let Some(next) = day.succ_opt() else {
            break;
        };
        let from = midnight(day);
        let until = midnight(next).min(at);
        let usd = costs
            .iter()
            .map(|cost| {
                let start = if cost.starts_at >= from
                    && cost.starts_at < midnight(next)
                    && cost.starts_at <= at
                {
                    cost.start_usd
                } else {
                    Decimal::ZERO
                };
                start
                    + elapsed_minutes(
                        cost.starts_at.max(from),
                        cost.ends_at.unwrap_or(at).min(until),
                    ) * cost.minute_usd
            })
            .sum();
        days.push(DailyBoatCost { day, usd });
        day = next;
    }
    days
}

/// Compute signed daily cost corrections without replaying a sandbox's start fee.
pub fn boat_cost_deltas(
    previous: Option<&AccountedBoat>,
    current: &AccountedBoat,
    range: DateRange,
) -> Vec<DailyBoatCost> {
    let mut changes: BTreeMap<_, Decimal> = BTreeMap::new();
    if let Some(previous) = previous {
        for day in daily_boat_costs(
            std::slice::from_ref(&previous.cost),
            range,
            previous.observed_at,
        ) {
            *changes.entry(day.day).or_default() -= day.usd;
        }
    }
    for day in daily_boat_costs(
        std::slice::from_ref(&current.cost),
        range,
        current.observed_at,
    ) {
        *changes.entry(day.day).or_default() += day.usd;
    }
    changes
        .into_iter()
        .filter(|(_, usd)| *usd != Decimal::ZERO)
        .map(|(day, usd)| DailyBoatCost { day, usd })
        .collect()
}

#[cfg(test)]
#[path = "../_tests_/boat_tests.rs"]
mod boat_tests;
