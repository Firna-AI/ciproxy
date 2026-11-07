//! Hand-worked end-to-end savings using retained per-size totals.

use chrono::Duration;
use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::MachineCost;
use ciproxy_store_interface::metering::totals::{DailyTotal, LocationTotals, SizeTotal};
use uuid::Uuid;

use crate::report::catalog::{Alternative, Comparison};
use crate::report::costs::{CostGap, Costs};
use crate::report::savings::{ReportInput, Saving, savings_report};
use crate::test_support::{at, catalog, day, free_history, range, size, usd};

#[test]
fn full_month_fixture_matches_hand_worked_costs_and_savings() {
    let days: Vec<_> = (0..30)
        .map(|offset| DailyTotal {
            day: day("2026-11-01") + Duration::days(offset),
            boat_usd: usd("0"),
            sizes: vec![SizeTotal {
                size: size(),
                machines: LocationTotals {
                    starts: 1728,
                    minutes: usd("8640"),
                },
                boat: LocationTotals::default(),
            }],
        })
        .collect();
    let machine = MachineRegistration {
        machine_id: Uuid::from_u128(1),
        registered_at: Some(at("2026-11-01T00:00:00Z")),
        removed_at: None,
    };
    let mut costs = free_history();
    costs.machines.push(MachineCost {
        id: Uuid::from_u128(2),
        machine_id: machine.machine_id,
        starts_at: at("2026-11-01T00:00:00Z"),
        ends_at: None,
        monthly_usd: usd("119"),
    });
    costs.plans[0].monthly_usd = usd("79");
    let lists = [
        catalog(Alternative::Blacksmith, "0.004"),
        catalog(Alternative::Github, "0.006"),
    ];
    let report = savings_report(ReportInput {
        range: range("2026-11-01", "2026-12-01"),
        at: at("2026-12-01T00:00:00Z"),
        days: &days,
        machines: &[machine],
        costs: &costs,
        price_lists: &lists,
    });
    assert_eq!(
        (
            report.machines.starts,
            report.machines.minutes,
            report.costs.total_usd
        ),
        (51840, usd("259200"), Some(usd("198")))
    );
    assert_eq!(
        (report.comparisons[0].usd, report.comparisons[1].usd),
        (Some(usd("1036.8")), Some(usd("1555.2")))
    );
    assert_eq!(
        (report.savings[0].usd, report.savings[1].usd, report.lead),
        (
            Some(usd("838.8")),
            Some(usd("1357.2")),
            Some(Alternative::Blacksmith)
        )
    );
    assert_eq!(report.comparisons[0].dated, day("2026-11-04"));
}

#[test]
fn negative_savings_keep_their_sign_and_unknown_comparison_hides_lead() {
    let days = vec![DailyTotal {
        day: day("2026-11-04"),
        boat_usd: usd("100"),
        sizes: vec![SizeTotal {
            size: size(),
            machines: LocationTotals {
                starts: 2,
                minutes: usd("10"),
            },
            boat: LocationTotals {
                starts: 1,
                minutes: usd("5"),
            },
        }],
    }];
    let costs = free_history();
    let lists = [
        catalog(Alternative::Blacksmith, "0.2"),
        catalog(Alternative::Github, "0.4"),
    ];
    let input = |price_lists| ReportInput {
        range: range("2026-11-04", "2026-11-05"),
        at: at("2026-11-05T00:00:00Z"),
        days: &days,
        machines: &[],
        costs: &costs,
        price_lists,
    };
    let report = savings_report(input(&lists));
    assert_eq!(
        (report.savings[0].usd, report.savings[1].usd),
        (Some(usd("-97")), Some(usd("-94")))
    );
    assert_eq!(
        (
            report.machines.starts,
            report.boat.starts,
            report.boat.minutes
        ),
        (2, 1, usd("5"))
    );
    let mut unknown = lists.clone();
    unknown[1].sizes.clear();
    let incomplete = savings_report(input(&unknown));
    assert_eq!(
        (
            incomplete.lead,
            incomplete.comparisons[1].usd,
            incomplete.savings[1].usd
        ),
        (None, None, None)
    );
    assert_eq!(incomplete.comparisons[1].missing[0].size, size());
}

#[test]
fn missing_machine_price_hides_both_savings_and_keeps_known_costs() {
    let days = vec![DailyTotal {
        day: day("2026-11-04"),
        boat_usd: usd("2"),
        sizes: vec![SizeTotal {
            size: size(),
            machines: LocationTotals {
                starts: 2,
                minutes: usd("10"),
            },
            boat: LocationTotals {
                starts: 1,
                minutes: usd("5"),
            },
        }],
    }];
    let machine = MachineRegistration {
        machine_id: Uuid::from_u128(1),
        registered_at: Some(at("2026-11-01T00:00:00Z")),
        removed_at: None,
    };
    let mut costs = free_history();
    costs.plans[0].monthly_usd = usd("90");
    let lists = [
        catalog(Alternative::Blacksmith, "0.2"),
        catalog(Alternative::Github, "0.4"),
    ];
    let report = savings_report(ReportInput {
        range: range("2026-11-04", "2026-11-05"),
        at: at("2026-11-05T00:00:00Z"),
        days: &days,
        machines: std::slice::from_ref(&machine),
        costs: &costs,
        price_lists: &lists,
    });
    assert_eq!(
        report.savings,
        [
            Saving {
                alternative: Alternative::Blacksmith,
                usd: None,
            },
            Saving {
                alternative: Alternative::Github,
                usd: None,
            },
        ]
    );
    assert_eq!(
        report.costs,
        Costs {
            known_machines_usd: usd("0"),
            boat_usd: usd("2"),
            known_plan_usd: usd("3"),
            total_usd: None,
            missing: vec![CostGap::MachinePrice {
                machine_id: machine.machine_id,
                day: day("2026-11-04"),
            }],
        }
    );
    assert_eq!(
        report.comparisons,
        [
            Comparison {
                alternative: Alternative::Blacksmith,
                dated: day("2026-11-04"),
                usd: Some(usd("3")),
                missing: vec![],
            },
            Comparison {
                alternative: Alternative::Github,
                dated: day("2026-11-04"),
                usd: Some(usd("6")),
                missing: vec![],
            },
        ]
    );
}
