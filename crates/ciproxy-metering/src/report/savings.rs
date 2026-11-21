//! Complete workspace savings reports from anonymous daily totals and histories.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::CostHistory;
use ciproxy_store_interface::metering::totals::{DailyTotal, LocationTotals};
use rust_decimal::Decimal;

use crate::report::catalog::{Alternative, Comparison, PriceList, compare};
use crate::report::costs::{Costs, calculate_costs};
use crate::report::range::DateRange;

/// All explicit inputs to the pure savings calculation.
pub struct ReportInput<'a> {
    /// The requested UTC report dates.
    pub range: DateRange,
    /// The explicit runtime observation time.
    pub at: DateTime<Utc>,
    /// Retained per-size daily totals, possibly older than visible job history.
    pub days: &'a [DailyTotal],
    /// Current and removed own-machine registration facts.
    pub machines: &'a [MachineRegistration],
    /// Customer, billing, and Boat cost histories.
    pub costs: &'a CostHistory,
    /// The dated Blacksmith and GitHub price lists.
    pub price_lists: &'a [PriceList; 2],
}

/// The signed savings against one alternative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Saving {
    /// The compared alternative.
    pub alternative: Alternative,
    /// Alternative cost minus ciproxy cost, negative when ciproxy costs more.
    pub usd: Option<Decimal>,
}

/// Savings, daily starts, runtime, and visible cost components of a workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavingsReport {
    /// The UTC date boundaries used for this report.
    pub range: DateRange,
    /// Starts and elapsed minutes on own machines.
    pub machines: LocationTotals,
    /// Starts and elapsed minutes on Boat.
    pub boat: LocationTotals,
    /// Daily per-size totals retained independently of job details.
    pub days: Vec<DailyTotal>,
    /// Ciproxy cost components and any missing monthly costs.
    pub costs: Costs,
    /// Each dated alternative's total and catalog gaps.
    pub comparisons: [Comparison; 2],
    /// Signed savings, left unknown when the underlying cost is incomplete.
    pub savings: [Saving; 2],
    /// The cheaper alternative, only when both comparison totals are known.
    pub lead: Option<Alternative>,
}

/// Compute savings across the entire workspace without any IO or clock reads.
pub fn savings_report(input: ReportInput<'_>) -> SavingsReport {
    let days: Vec<_> = input
        .days
        .iter()
        .filter(|day| {
            day.day >= input.range.from
                && day.day < input.range.until
                && day.day <= input.at.date_naive()
        })
        .cloned()
        .collect();
    let mut machines = LocationTotals::default();
    let mut boat = LocationTotals::default();
    for size in days.iter().flat_map(|day| &day.sizes) {
        machines.starts += size.machines.starts;
        machines.minutes += size.machines.minutes;
        boat.starts += size.boat.starts;
        boat.minutes += size.boat.minutes;
    }
    let boat_usd = days.iter().map(|day| day.boat_usd).sum();
    let costs = calculate_costs(input.machines, input.costs, input.range, input.at, boat_usd);
    let comparisons = input
        .price_lists
        .each_ref()
        .map(|list| compare(list, &days));
    let savings = comparisons.each_ref().map(|comparison| Saving {
        alternative: comparison.alternative,
        usd: comparison
            .usd
            .zip(costs.total_usd)
            .map(|(alternative, total)| alternative - total),
    });
    let lead = comparisons[0]
        .usd
        .zip(comparisons[1].usd)
        .map(|(first, second)| {
            if first <= second {
                comparisons[0].alternative
            } else {
                comparisons[1].alternative
            }
        });
    SavingsReport {
        range: input.range,
        machines,
        boat,
        days,
        costs,
        comparisons,
        savings,
        lead,
    }
}

#[cfg(test)]
#[path = "../_tests_/savings_tests.rs"]
mod savings_tests;
