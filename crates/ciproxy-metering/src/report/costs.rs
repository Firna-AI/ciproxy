//! Registered-day monthly costs and elapsed Boat sandbox costs.

use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::CostHistory;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::report::range::{DateRange, midnight};

/// Missing facts that prevent an accurate ciproxy total.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CostGap {
    /// Legacy workspace creation time has not been supplied.
    UnknownWorkspaceStart,
    /// A legacy machine has no trusted registration time.
    UnknownRegistration(Uuid),
    /// A registered day has no known final monthly machine price.
    MachinePrice {
        /// The own machine that needs a cost.
        machine_id: Uuid,
        /// The UTC registration date without a known final price.
        day: NaiveDate,
    },
    /// A report date has no known final monthly tier and add-on price.
    PlanPrice(NaiveDate),
}

/// Costs remain visible even when missing machine costs suppress savings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Costs {
    /// The sum of the known own-machine registered-day costs.
    pub known_machines_usd: Decimal,
    /// Boat start charges and elapsed runtime costs, including idle sandboxes.
    pub boat_usd: Decimal,
    /// The sum of the known tier and add-on day costs.
    pub known_plan_usd: Decimal,
    /// The full ciproxy cost only when every required monthly cost is known.
    pub total_usd: Option<Decimal>,
    /// Facts to request from the customer or trusted history producer.
    pub missing: Vec<CostGap>,
}

/// Allocate monthly costs to registration days and sum ciproxy Boat costs.
///
/// Each registration day counts once, including partial registration days.
/// The last monthly price on a UTC date applies to that whole date.
/// Each date uses the day count of its calendar month for both price kinds.
pub fn calculate_costs(
    machines: &[MachineRegistration],
    history: &CostHistory,
    range: DateRange,
    at: DateTime<Utc>,
    boat_usd: Decimal,
) -> Costs {
    let mut machine_rates: BTreeMap<(Decimal, u32), u32> = BTreeMap::new();
    let mut plan_rates: BTreeMap<(Decimal, u32), u32> = BTreeMap::new();
    let mut missing = Vec::new();
    if history.workspace_started_at.is_none() {
        missing.push(CostGap::UnknownWorkspaceStart);
    }
    let mut day = range.from;
    while day < range.until && day <= at.date_naive() {
        let Some(next) = day.succ_opt() else {
            break;
        };
        let from = midnight(day);
        let until = midnight(next);
        if history
            .workspace_started_at
            .is_some_and(|start| start >= until)
        {
            day = next;
            continue;
        }
        for machine in machines {
            if machine.removed_at.is_some_and(|removed| removed <= from) {
                continue;
            }
            let Some(registered) = machine.registered_at else {
                let gap = CostGap::UnknownRegistration(machine.machine_id);
                if !missing.contains(&gap) {
                    missing.push(gap);
                }
                continue;
            };
            if registered >= until {
                continue;
            }
            let prices: Vec<_> = history
                .machines
                .iter()
                .filter(|price| {
                    price.machine_id == machine.machine_id
                        && price.starts_at < until
                        && price.ends_at.is_none_or(|end| end > from)
                })
                .map(|price| (price.starts_at, price.ends_at, price.monthly_usd))
                .collect();
            if let Some(monthly) = last_price(prices) {
                *machine_rates
                    .entry((monthly, days_in_month(day)))
                    .or_default() += 1;
            } else {
                missing.push(CostGap::MachinePrice {
                    machine_id: machine.machine_id,
                    day,
                });
            }
        }
        let prices: Vec<_> = history
            .plans
            .iter()
            .filter(|price| price.starts_at < until && price.ends_at.is_none_or(|end| end > from))
            .map(|price| (price.starts_at, price.ends_at, price.monthly_usd))
            .collect();
        if let Some(monthly) = last_price(prices) {
            *plan_rates.entry((monthly, days_in_month(day))).or_default() += 1;
        } else {
            missing.push(CostGap::PlanPrice(day));
        }
        day = next;
    }
    let known_machines_usd = monthly_total(machine_rates);
    let known_plan_usd = monthly_total(plan_rates);
    Costs {
        known_machines_usd,
        boat_usd,
        known_plan_usd,
        total_usd: missing
            .is_empty()
            .then_some(known_machines_usd + known_plan_usd + boat_usd),
        missing,
    }
}

/// Normal price changes form a continuous chain, independent of input order.
/// Malformed overlapping or disconnected histories remain incomplete.
fn last_price(mut prices: Vec<(DateTime<Utc>, Option<DateTime<Utc>>, Decimal)>) -> Option<Decimal> {
    prices.sort_by_key(|(starts, _, _)| *starts);
    if prices.windows(2).any(|pair| pair[0].1 != Some(pair[1].0)) {
        return None;
    }
    prices.last().map(|(_, _, monthly)| *monthly)
}

fn days_in_month(day: NaiveDate) -> u32 {
    match day.month() {
        4 | 6 | 9 | 11 => 30,
        2 if NaiveDate::from_ymd_opt(day.year(), 2, 29).is_some() => 29,
        2 => 28,
        _ => 31,
    }
}

fn monthly_total(rates: BTreeMap<(Decimal, u32), u32>) -> Decimal {
    rates
        .into_iter()
        .map(|((monthly, divisor), days)| monthly * Decimal::from(days) / Decimal::from(divisor))
        .sum()
}

#[cfg(test)]
#[path = "../_tests_/costs_tests.rs"]
mod costs_tests;

#[cfg(test)]
#[path = "../_tests_/monthly_costs_tests.rs"]
mod monthly_costs_tests;
