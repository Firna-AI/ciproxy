//! Savings wire conversion after exact store-backed metering calculations.

use ciproxy_metering::report::catalog::{Alternative, PriceMatch};
use ciproxy_metering::report::costs::CostGap;
use ciproxy_metering::report::savings::SavingsReport;
use ciproxy_store_interface::metering::totals::LocationTotals;

use crate::wire::jobs::JobVmSize;
use crate::wire::savings::{
    SavingsAlternative, SavingsComparison, SavingsCostGap, SavingsDay, SavingsPage,
    SavingsPriceGap, SavingsPriceGapReason, SavingsRange, SavingsRuntime,
};

use super::billing_views::cents;

pub(super) fn report(
    report: SavingsReport,
    range: SavingsRange,
    refresh_complete: bool,
) -> SavingsPage {
    let mut days = Vec::new();
    let mut day = report.range.from;
    while day < report.range.until {
        let mut machines = LocationTotals::default();
        let mut boat = LocationTotals::default();
        for size in report
            .days
            .iter()
            .filter(|total| total.day == day)
            .flat_map(|total| &total.sizes)
        {
            machines.starts += size.machines.starts;
            machines.minutes += size.machines.minutes;
            boat.starts += size.boat.starts;
            boat.minutes += size.boat.minutes;
        }
        days.push(SavingsDay {
            day,
            machines: runtime(machines),
            boat: runtime(boat),
        });
        let Some(next) = day.succ_opt() else {
            break;
        };
        day = next;
    }
    let comparisons = report
        .comparisons
        .into_iter()
        .zip(report.savings)
        .map(|(comparison, saving)| SavingsComparison {
            alternative: alternative(comparison.alternative),
            price_list_date: comparison.dated,
            cost_cents: comparison.usd.map(cents),
            savings_cents: saving.usd.map(cents),
            missing_prices: comparison
                .missing
                .into_iter()
                .map(|missing| SavingsPriceGap {
                    size: JobVmSize {
                        vcpus: missing.size.vcpu,
                        memory_bytes: missing.size.memory_bytes.to_string(),
                        disk_bytes: missing.size.disk_bytes.to_string(),
                    },
                    reason: match missing.price_match {
                        PriceMatch::Ambiguous(_) => SavingsPriceGapReason::Ambiguous,
                        _ => SavingsPriceGapReason::Unavailable,
                    },
                })
                .collect(),
        })
        .collect();
    SavingsPage {
        range,
        from: report.range.from,
        until: report.range.until,
        machines: runtime(report.machines),
        boat: runtime(report.boat),
        days,
        known_machine_cents: cents(report.costs.known_machines_usd),
        boat_cents: cents(report.costs.boat_usd),
        known_plan_cents: cents(report.costs.known_plan_usd),
        total_cents: report.costs.total_usd.map(cents),
        missing_costs: report
            .costs
            .missing
            .into_iter()
            .map(|gap| match gap {
                CostGap::UnknownWorkspaceStart => SavingsCostGap::WorkspaceStart,
                CostGap::UnknownRegistration(id) => SavingsCostGap::MachineRegistration {
                    machine_id: id.to_string(),
                },
                CostGap::MachinePrice { machine_id, day } => SavingsCostGap::MachinePrice {
                    machine_id: machine_id.to_string(),
                    day,
                },
                CostGap::PlanPrice(day) => SavingsCostGap::PlanPrice { day },
            })
            .collect(),
        comparisons,
        lead: report.lead.map(alternative),
        can_add_machine_costs: false,
        refresh_complete,
        banner: None,
    }
}

fn runtime(value: LocationTotals) -> SavingsRuntime {
    SavingsRuntime {
        starts: value.starts.to_string(),
        minutes: value.minutes.normalize().to_string(),
    }
}

fn alternative(value: Alternative) -> SavingsAlternative {
    match value {
        Alternative::Blacksmith => SavingsAlternative::Blacksmith,
        Alternative::Github => SavingsAlternative::Github,
    }
}
