//! Hand-worked registration-day costs and explicit missing-history behavior.

use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::{MachineCost, PlanCost};
use uuid::Uuid;

use crate::report::costs::{CostGap, calculate_costs};
use crate::test_support::{at, day, free_history, range, usd};

#[test]
fn registration_days_and_plan_price_history_count_the_full_range() {
    let machine = MachineRegistration {
        machine_id: Uuid::from_u128(1),
        registered_at: Some(at("2026-11-04T12:00:00Z")),
        removed_at: Some(at("2026-11-05T12:00:00Z")),
    };
    let mut history = free_history();
    history.machines.push(MachineCost {
        id: Uuid::from_u128(2),
        machine_id: machine.machine_id,
        starts_at: at("2026-11-04T12:00:00Z"),
        ends_at: None,
        monthly_usd: usd("60"),
    });
    history.plans = vec![
        PlanCost {
            id: Uuid::from_u128(3),
            starts_at: at("2026-11-01T00:00:00Z"),
            ends_at: Some(at("2026-11-05T00:00:00Z")),
            monthly_usd: usd("60"),
        },
        PlanCost {
            id: Uuid::from_u128(4),
            starts_at: at("2026-11-05T00:00:00Z"),
            ends_at: None,
            monthly_usd: usd("90"),
        },
    ];
    let costs = calculate_costs(
        &[machine],
        &history,
        range("2026-11-04", "2026-11-07"),
        at("2026-11-06T23:00:00Z"),
        usd("3"),
    );
    assert_eq!(
        (
            costs.known_machines_usd,
            costs.known_plan_usd,
            costs.boat_usd,
            costs.total_usd
        ),
        (usd("4"), usd("8"), usd("3"), Some(usd("15")))
    );
}

#[test]
fn calendar_divisor_changes_between_months_without_cent_rounding() {
    let machine = MachineRegistration {
        machine_id: Uuid::from_u128(1),
        registered_at: Some(at("2026-01-01T00:00:00Z")),
        removed_at: None,
    };
    let mut history = free_history();
    history.machines.push(MachineCost {
        id: Uuid::from_u128(2),
        machine_id: machine.machine_id,
        starts_at: at("2026-01-01T00:00:00Z"),
        ends_at: None,
        monthly_usd: usd("868"),
    });
    let costs = calculate_costs(
        &[machine],
        &history,
        range("2026-01-31", "2026-02-02"),
        at("2026-02-02T00:00:00Z"),
        usd("0"),
    );
    assert_eq!(costs.total_usd, Some(usd("59")));
}

#[test]
fn missing_machine_costs_keep_other_costs_visible() {
    let machine = MachineRegistration {
        machine_id: Uuid::from_u128(1),
        registered_at: Some(at("2026-11-04T00:00:00Z")),
        removed_at: None,
    };
    let history = free_history();
    let costs = calculate_costs(
        std::slice::from_ref(&machine),
        &history,
        range("2026-11-04", "2026-11-05"),
        at("2026-11-04T12:00:00Z"),
        usd("3"),
    );
    assert_eq!(
        (costs.total_usd, costs.boat_usd, costs.known_plan_usd),
        (None, usd("3"), usd("0"))
    );
    assert_eq!(
        costs.missing,
        vec![CostGap::MachinePrice {
            machine_id: machine.machine_id,
            day: day("2026-11-04")
        }]
    );
    let unknown = MachineRegistration {
        registered_at: None,
        ..machine
    };
    let costs = calculate_costs(
        &[unknown],
        &history,
        range("2026-11-04", "2026-11-05"),
        at("2026-11-04T12:00:00Z"),
        usd("3"),
    );
    assert_eq!(
        (costs.total_usd, costs.missing),
        (None, vec![CostGap::UnknownRegistration(Uuid::from_u128(1))])
    );
}

#[test]
fn new_workspace_has_zero_plan_cost_before_creation() {
    let mut history = free_history();
    history.workspace_started_at = Some(at("2026-11-04T12:00:00Z"));
    history.plans[0].starts_at = at("2026-11-04T12:00:00Z");
    let costs = calculate_costs(
        &[],
        &history,
        range("2026-10-06", "2026-11-05"),
        at("2026-11-04T15:00:00Z"),
        usd("0"),
    );
    assert_eq!((costs.total_usd, costs.missing), (Some(usd("0")), vec![]));
}

#[test]
fn missing_plan_history_stays_incomplete() {
    let mut history = free_history();
    history.plans.clear();
    let costs = calculate_costs(
        &[],
        &history,
        range("2026-11-04", "2026-11-05"),
        at("2026-11-04T15:00:00Z"),
        usd("3"),
    );
    assert_eq!(
        (costs.total_usd, costs.boat_usd, costs.missing),
        (None, usd("3"), vec![CostGap::PlanPrice(day("2026-11-04"))])
    );
}
