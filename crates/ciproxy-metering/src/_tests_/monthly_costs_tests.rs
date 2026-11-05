//! Calendar-day monthly costs with literal, hand-worked results.

use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::{CostHistory, MachineCost, PlanCost};
use uuid::Uuid;

use crate::report::costs::calculate_costs;
use crate::report::range::{ReportRange, report_range};
use crate::test_support::{at, free_history, range, usd};

#[test]
fn last_monthly_prices_apply_to_the_full_change_date_in_any_input_order() {
    let machine = registered_machine("2026-01-01T00:00:00Z");
    let change = at("2026-03-10T15:00:00Z");
    let tomorrow = at("2026-03-11T00:00:00Z");
    let mut history = monthly_history(&machine, "31", "79");
    history.machines[0].ends_at = Some(change);
    history.machines.extend([
        MachineCost {
            id: Uuid::from_u128(3),
            starts_at: change,
            ends_at: Some(tomorrow),
            monthly_usd: usd("62"),
            ..history.machines[0].clone()
        },
        MachineCost {
            id: Uuid::from_u128(4),
            starts_at: tomorrow,
            ends_at: None,
            monthly_usd: usd("93"),
            ..history.machines[0].clone()
        },
    ]);
    history.plans[0].ends_at = Some(change);
    history.plans.extend([
        PlanCost {
            id: Uuid::from_u128(5),
            starts_at: change,
            ends_at: Some(tomorrow),
            monthly_usd: usd("319"),
        },
        PlanCost {
            id: Uuid::from_u128(6),
            starts_at: tomorrow,
            ends_at: None,
            monthly_usd: usd("339"),
        },
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
            range("2026-03-10", "2026-03-11"),
            at("2026-03-10T23:59:59Z"),
            usd("0"),
        );
        assert_eq!(
            (
                costs.known_machines_usd,
                costs.known_plan_usd,
                costs.total_usd,
                costs.missing,
            ),
            (
                usd("2"),
                usd("10.290322580645161290322580645"),
                Some(usd("12.290322580645161290322580645")),
                vec![],
            ),
            "reverse input: {reverse}",
        );
    }
}

#[test]
fn complete_calendar_months_count_each_monthly_price_exactly_once() {
    let machine = registered_machine("2026-01-01T00:00:00Z");
    let history = monthly_history(&machine, "119.03", "79.01");
    for (from, until) in [
        ("2026-03-01", "2026-04-01"),
        ("2026-02-01", "2026-03-01"),
        ("2028-02-01", "2028-03-01"),
    ] {
        let dates = range(from, until);
        let costs = calculate_costs(
            std::slice::from_ref(&machine),
            &history,
            dates,
            at("2028-03-01T00:00:00Z"),
            usd("0"),
        );
        assert_eq!(
            (
                costs.known_machines_usd,
                costs.known_plan_usd,
                costs.total_usd,
                costs.missing,
            ),
            (usd("119.03"), usd("79.01"), Some(usd("198.04")), vec![]),
            "month beginning {from}",
        );
    }
}

#[test]
fn last_thirty_dates_use_both_calendar_month_lengths() {
    let machine = registered_machine("2026-01-01T00:00:00Z");
    let history = monthly_history(&machine, "868", "434");
    let observed = at("2026-03-16T23:00:00Z");
    let dates = report_range(ReportRange::Last30Days, observed).unwrap();
    let costs = calculate_costs(&[machine], &history, dates, observed, usd("0"));
    assert_eq!(dates, range("2026-02-15", "2026-03-17"));
    assert_eq!(
        (
            costs.known_machines_usd,
            costs.known_plan_usd,
            costs.total_usd,
            costs.missing,
        ),
        (usd("882"), usd("441"), Some(usd("1323")), vec![]),
    );
}

#[test]
fn mid_month_registration_counts_its_partial_first_date() {
    let machine = registered_machine("2026-03-16T15:00:00Z");
    let mut history = monthly_history(&machine, "310", "31");
    history.machines[0].starts_at = machine.registered_at.unwrap();
    let costs = calculate_costs(
        &[machine],
        &history,
        range("2026-03-01", "2026-04-01"),
        at("2026-04-01T00:00:00Z"),
        usd("0"),
    );
    assert_eq!(
        (
            costs.known_machines_usd,
            costs.known_plan_usd,
            costs.total_usd,
            costs.missing,
        ),
        (usd("160"), usd("31"), Some(usd("191")), vec![]),
    );
}

fn registered_machine(registered: &str) -> MachineRegistration {
    MachineRegistration {
        machine_id: Uuid::from_u128(1),
        registered_at: Some(at(registered)),
        removed_at: None,
    }
}

fn monthly_history(
    machine: &MachineRegistration,
    machine_usd: &str,
    plan_usd: &str,
) -> CostHistory {
    let mut history = free_history();
    history.machines.push(MachineCost {
        id: Uuid::from_u128(2),
        machine_id: machine.machine_id,
        starts_at: at("2026-01-01T00:00:00Z"),
        ends_at: None,
        monthly_usd: usd(machine_usd),
    });
    history.plans[0].monthly_usd = usd(plan_usd);
    history
}
