//! Anonymous daily costs and incremental corrections for each provider.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::metering::history::OverflowCost;
use ciproxy_store_interface::metering::totals::{AccountedOverflow, DailyOverflowCost};
use rust_decimal::Decimal;

use crate::report::range::{DateRange, midnight};
use crate::report::runtime::elapsed_minutes;

/// Allocate start charges to their start date and runtime to UTC day overlap.
pub fn daily_overflow_costs(
    costs: &[OverflowCost],
    range: DateRange,
    at: DateTime<Utc>,
) -> Vec<DailyOverflowCost> {
    let mut days = Vec::new();
    let mut day = range.from;
    while day < range.until && day <= at.date_naive() {
        let Some(next) = day.succ_opt() else {
            break;
        };
        let from = midnight(day);
        let until = midnight(next).min(at);
        let mut providers = BTreeMap::new();
        for cost in costs {
            let start = if cost.starts_at >= from
                && cost.starts_at < midnight(next)
                && cost.starts_at <= at
            {
                cost.start_usd
            } else {
                Decimal::ZERO
            };
            let minutes = elapsed_minutes(
                cost.starts_at.max(from),
                cost.ends_at.unwrap_or(at).min(until),
            );
            *providers.entry(cost.provider).or_insert(Decimal::ZERO) +=
                start + minutes * cost.minute_usd;
        }
        days.extend(
            providers
                .into_iter()
                .map(|(provider, usd)| DailyOverflowCost { day, provider, usd }),
        );
        day = next;
    }
    days
}

/// Compute signed daily cost corrections without replaying a sandbox's start fee.
pub fn overflow_cost_deltas(
    previous: Option<&AccountedOverflow>,
    current: &AccountedOverflow,
    range: DateRange,
) -> Vec<DailyOverflowCost> {
    let mut changes: BTreeMap<_, Decimal> = BTreeMap::new();
    if let Some(previous) = previous {
        for day in daily_overflow_costs(
            std::slice::from_ref(&previous.cost),
            range,
            previous.observed_at,
        ) {
            *changes.entry((day.day, day.provider)).or_default() -= day.usd;
        }
    }
    for day in daily_overflow_costs(
        std::slice::from_ref(&current.cost),
        range,
        current.observed_at,
    ) {
        *changes.entry((day.day, day.provider)).or_default() += day.usd;
    }
    changes
        .into_iter()
        .filter(|(_, usd)| *usd != Decimal::ZERO)
        .map(|((day, provider), usd)| DailyOverflowCost { day, provider, usd })
        .collect()
}

#[cfg(test)]
#[path = "../_tests_/overflow_tests.rs"]
mod overflow_tests;
