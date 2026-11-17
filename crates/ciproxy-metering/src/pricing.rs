//! Published tier capacities, manually selected add-ons, and paid offers.

use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// Capacity and monthly price of a tier with add-ons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanChoice {
    /// The tier and required add-on quantities.
    pub plan: Selection,
    /// The included starts of each billing period.
    pub starts: Decimal,
    /// The included current machine count.
    pub machines: u128,
    /// The monthly tier and add-on total in USD.
    pub monthly_usd: Decimal,
}

/// Compute the published capacities and price without changing the saved plan.
pub fn plan_choice(mut plan: Selection) -> PlanChoice {
    if plan.tier == Tier::Free {
        plan.add_ons = AddOns::default();
    }
    PlanChoice {
        plan,
        starts: Decimal::from(plan.included_starts()),
        machines: u128::from(plan.included_machines()),
        monthly_usd: Decimal::from(plan.monthly_cents()) / Decimal::from(100),
    }
}

/// Find the cheapest paid tier and smallest add-ons that cover projected use.
///
/// Equal prices prefer the current paid tier, then Team.
pub fn cheapest_offer(expected_starts: Decimal, machines: u64, current: Tier) -> PlanChoice {
    let team = covering(Tier::Team, expected_starts, machines);
    let business = covering(Tier::Business, expected_starts, machines);
    if business.monthly_usd < team.monthly_usd
        || (business.monthly_usd == team.monthly_usd && current == Tier::Business)
    {
        business
    } else {
        team
    }
}

fn covering(tier: Tier, expected: Decimal, machine_count: u64) -> PlanChoice {
    let blocks = ((expected - Decimal::from(tier.included_starts())).max(Decimal::ZERO)
        / Decimal::from(AddOns::STARTS_PER_BLOCK))
    .ceil()
    .to_u32()
    .unwrap_or(u32::MAX);
    plan_choice(Selection {
        tier,
        add_ons: AddOns {
            start_blocks: blocks,
            machines: machine_count
                .saturating_sub(tier.included_machines())
                .try_into()
                .unwrap_or(u32::MAX),
        },
    })
}

#[cfg(test)]
#[path = "_tests_/pricing_tests.rs"]
mod pricing_tests;
