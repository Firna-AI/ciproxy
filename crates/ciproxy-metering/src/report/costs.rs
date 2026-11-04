//! Registered-day monthly costs and elapsed Boat sandbox costs.

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::CostHistory;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::report::range::{DateRange, midnight};

/// Explicit denominator for a monthly cost's registered-day allocation.
///
/// The protocol supplies no default. The composition root must select it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonthlyDivisor {
    /// Divide each day's monthly rate by that UTC calendar month's length.
    CalendarMonthDays,
    /// Divide each day's monthly rate by an explicitly supplied day count.
    FixedDays(NonZeroU32),
}

/// Missing facts that prevent an accurate ciproxy total.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CostGap {
    /// Legacy workspace creation time has not been supplied.
    UnknownWorkspaceStart,
    /// A legacy machine has no trusted registration time.
    UnknownRegistration(Uuid),
    /// A registered day has no single known monthly machine price.
    MachinePrice {
        /// The own machine that needs a cost.
        machine_id: Uuid,
        /// The UTC registration date without a unique price.
        day: NaiveDate,
    },
    /// A report date has no single confirmed monthly tier and add-on price.
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
/// A monthly price must uniquely cover each counted day. Overlapping prices
/// stay incomplete rather than choosing an unapproved same-day change rule.
pub fn calculate_costs(
    machines: &[MachineRegistration],
    history: &CostHistory,
    range: DateRange,
    at: DateTime<Utc>,
    divisor: MonthlyDivisor,
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
                .collect();
            if let [price] = prices.as_slice() {
                *machine_rates
                    .entry((price.monthly_usd, day_divisor(divisor, day)))
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
            .collect();
        if let [price] = prices.as_slice() {
            *plan_rates
                .entry((price.monthly_usd, day_divisor(divisor, day)))
                .or_default() += 1;
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

fn day_divisor(divisor: MonthlyDivisor, day: NaiveDate) -> u32 {
    match divisor {
        MonthlyDivisor::FixedDays(days) => days.get(),
        MonthlyDivisor::CalendarMonthDays => match day.month() {
            4 | 6 | 9 | 11 => 30,
            2 if NaiveDate::from_ymd_opt(day.year(), 2, 29).is_some() => 29,
            2 => 28,
            _ => 31,
        },
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
