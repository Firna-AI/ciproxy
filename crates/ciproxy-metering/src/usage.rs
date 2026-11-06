//! Workspace counts, inclusive-day forecasts, and advisory soft limits.

use std::collections::BTreeSet;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use ciproxy_store_interface::metering::facts::WorkspaceUsage;
use ciproxy_store_interface::metering::history::Period;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::pricing::{Plan, PlanChoice, Tier, cheapest_offer, plan_choice};

/// The single banner that a page displays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Banner {
    /// Stripe retries a failed payment.
    PaymentFailed,
    /// The paid subscription ended and its notice remains visible.
    SubscriptionEnded,
    /// The workspace exceeds an advisory capacity.
    SoftLimit,
}

/// Payment notices supplied by billing, before soft-limit precedence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PaymentNotice {
    /// Billing has no notice to display.
    #[default]
    None,
    /// A paid subscription has a failed charge.
    Failed,
    /// Billing still displays the notice of an ended subscription.
    Ended,
}

/// All current use and advisory limits of a workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Usage {
    /// Runner-taken jobs in the current period.
    pub starts: u64,
    /// Current registrations, including offline or incapable machines.
    pub machines: u64,
    /// Active or suspended organizations in this workspace.
    pub organizations: u64,
    /// Projected starts using the current UTC day in elapsed days.
    pub expected_starts: Decimal,
    /// Starts above the included capacity, with zero at the exact limit.
    pub starts_over: Decimal,
    /// Machines above the included capacity.
    pub machines_over: u128,
    /// Organizations above Free's single-organization soft limit.
    pub organizations_over: u64,
    /// The cheapest paid choice, which never applies itself.
    pub offer: PlanChoice,
    /// The first UTC day projected strictly above the starts capacity.
    pub crossing_day: Option<NaiveDate>,
    /// The single notice after payment-notice precedence.
    pub banner: Option<Banner>,
}

/// Count workspace facts and calculate advisory pricing use.
pub fn workspace_usage(
    facts: &WorkspaceUsage,
    period: Period,
    plan: Plan,
    at: DateTime<Utc>,
    payment: PaymentNotice,
) -> Usage {
    let live: BTreeSet<_> = facts
        .jobs
        .iter()
        .filter(|job| job.started_at >= period.starts_at && job.started_at < period.ends_at)
        .filter(|job| job.started_at <= at)
        .map(|job| job.key())
        .collect();
    let within = |taken| taken >= period.starts_at && taken < period.ends_at && taken <= at;
    let recorded: i64 = facts
        .starts
        .iter()
        .filter(|total| within(total.taken_at))
        .map(|total| total.starts)
        .sum();
    let previous = facts
        .accounted_jobs
        .iter()
        .filter(|job| job.job.started_at <= job.observed_at)
        .filter(|job| within(job.job.started_at))
        .count() as i64;
    let starts = (recorded + live.len() as i64 - previous).max(0) as u64;
    let machines = facts
        .machines
        .iter()
        .filter(|machine| machine.removed_at.is_none())
        .map(|machine| machine.machine_id)
        .collect::<BTreeSet<_>>()
        .len() as u64;
    let last = period
        .ends_at
        .checked_sub_signed(Duration::nanoseconds(1))
        .unwrap_or(period.ends_at);
    let days = ((last.date_naive() - period.starts_at.date_naive()).num_days() + 1).max(1);
    let elapsed = ((at.date_naive() - period.starts_at.date_naive()).num_days() + 1).clamp(1, days);
    let expected_starts = Decimal::from(starts) * Decimal::from(days) / Decimal::from(elapsed);
    let included = plan_choice(plan);
    let starts_over = (Decimal::from(starts) - included.starts).max(Decimal::ZERO);
    let machines_over = u128::from(machines).saturating_sub(included.machines);
    let organizations_over = if plan.tier == Tier::Free {
        facts.organizations.saturating_sub(1)
    } else {
        0
    };
    let exceeded = starts_over > Decimal::ZERO || machines_over > 0 || organizations_over > 0;
    let banner = match payment {
        PaymentNotice::Failed => Some(Banner::PaymentFailed),
        PaymentNotice::Ended => Some(Banner::SubscriptionEnded),
        PaymentNotice::None if exceeded => Some(Banner::SoftLimit),
        PaymentNotice::None => None,
    };
    let crossing_day =
        if Decimal::from(starts) < included.starts && expected_starts > included.starts {
            let crossing = (included.starts * Decimal::from(elapsed) / Decimal::from(starts))
                .floor()
                .to_i64()
                .unwrap_or_default();
            period
                .starts_at
                .date_naive()
                .checked_add_signed(Duration::days(crossing))
        } else {
            None
        };
    Usage {
        starts,
        machines,
        organizations: facts.organizations,
        expected_starts,
        starts_over,
        machines_over,
        organizations_over,
        offer: cheapest_offer(expected_starts, machines, plan.tier),
        crossing_day,
        banner,
    }
}

#[cfg(test)]
#[path = "_tests_/usage_tests.rs"]
mod usage_tests;
