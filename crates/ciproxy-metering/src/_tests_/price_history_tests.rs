//! Price-interval regressions through the public cost calculation.

use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::{CostHistory, MachineCost, PlanCost};
use uuid::Uuid;

use crate::report::costs::{CostGap, calculate_costs};
use crate::test_support::{at, day, free_history, range, usd};

#[test]
fn overlapping_prices_keep_prior_known_costs_and_exact_gaps() {
    let machine = registered_machine();
    let history = price_history(&[
        ("2026-11-01T00:00:00Z", None, "60", "90"),
        ("2026-11-05T12:00:00Z", None, "120", "150"),
    ]);
    let costs = calculate_costs(
        &[machine],
        &history,
        range("2026-11-04", "2026-11-06"),
        at("2026-11-05T23:00:00Z"),
        usd("3"),
    );
    assert_eq!(
        (
            costs.known_machines_usd,
            costs.known_plan_usd,
            costs.boat_usd,
            costs.total_usd,
            costs.missing,
        ),
        (
            usd("2"),
            usd("3"),
            usd("3"),
            None,
            vec![
                CostGap::MachinePrice {
                    machine_id: Uuid::from_u128(1),
                    day: day("2026-11-05"),
                },
                CostGap::PlanPrice(day("2026-11-05")),
            ],
        ),
    );
}

#[test]
fn same_date_price_gaps_use_the_last_price_for_the_full_date() {
    let machine = registered_machine();
    let history = price_history(&[
        (
            "2026-11-01T00:00:00Z",
            Some("2026-11-05T09:00:00Z"),
            "60",
            "90",
        ),
        ("2026-11-05T15:00:00Z", None, "120", "150"),
    ]);
    let costs = calculate_costs(
        &[machine],
        &history,
        range("2026-11-04", "2026-11-06"),
        at("2026-11-05T23:00:00Z"),
        usd("1"),
    );
    assert_eq!(
        (
            costs.known_machines_usd,
            costs.known_plan_usd,
            costs.boat_usd,
            costs.total_usd,
            costs.missing,
        ),
        (usd("6"), usd("8"), usd("1"), Some(usd("15")), vec![]),
    );
}

#[test]
fn empty_prices_do_not_change_the_real_successor_in_either_input_order() {
    let machine = registered_machine();
    let history = price_history(&[
        (
            "2026-11-01T00:00:00Z",
            Some("2026-11-05T15:00:00Z"),
            "60",
            "90",
        ),
        (
            "2026-11-05T15:00:00Z",
            Some("2026-11-05T15:00:00Z"),
            "999",
            "999",
        ),
        ("2026-11-05T15:00:00Z", None, "120", "150"),
    ]);
    for reverse in [false, true] {
        let mut ordered = history.clone();
        if reverse {
            ordered.machines.reverse();
            ordered.plans.reverse();
        }
        let costs = calculate_costs(
            std::slice::from_ref(&machine),
            &ordered,
            range("2026-11-05", "2026-11-06"),
            at("2026-11-05T23:00:00Z"),
            usd("1"),
        );
        assert_eq!(
            (
                costs.known_machines_usd,
                costs.known_plan_usd,
                costs.boat_usd,
                costs.total_usd,
                costs.missing,
            ),
            (usd("4"), usd("5"), usd("1"), Some(usd("10")), vec![]),
            "reverse input: {reverse}",
        );
    }
}

fn registered_machine() -> MachineRegistration {
    MachineRegistration {
        machine_id: Uuid::from_u128(1),
        registered_at: Some(at("2026-11-01T00:00:00Z")),
        removed_at: None,
    }
}

fn price_history(prices: &[(&str, Option<&str>, &str, &str)]) -> CostHistory {
    let mut history = free_history();
    history.plans.clear();
    for ((starts, ends, machine_usd, plan_usd), id) in prices.iter().zip(1_u128..) {
        history.machines.push(MachineCost {
            id: Uuid::from_u128(id * 2),
            machine_id: Uuid::from_u128(1),
            starts_at: at(starts),
            ends_at: ends.map(at),
            monthly_usd: usd(machine_usd),
        });
        history.plans.push(PlanCost {
            id: Uuid::from_u128(id * 2 + 1),
            starts_at: at(starts),
            ends_at: ends.map(at),
            monthly_usd: usd(plan_usd),
        });
    }
    history
}
