//! Published tier capacities, manually selected add-ons, and paid offers.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// A published ciproxy pricing tier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tier {
    /// The tier with no subscription or add-ons.
    Free,
    /// The first paid tier.
    Team,
    /// The larger paid tier.
    Business,
}

/// Customer-selected monthly add-on quantities.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AddOns {
    /// Blocks of ten thousand extra starts.
    pub start_blocks: u64,
    /// Extra own machines.
    pub machines: u64,
}

/// A tier with its customer-selected add-ons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Plan {
    /// The published tier.
    pub tier: Tier,
    /// The manually saved quantities, ignored for Free.
    pub add_ons: AddOns,
}

/// Capacity and monthly price of a tier with add-ons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanChoice {
    /// The tier and required add-on quantities.
    pub plan: Plan,
    /// The included starts of each billing period.
    pub starts: Decimal,
    /// The included current machine count.
    pub machines: u128,
    /// The monthly tier and add-on total in USD.
    pub monthly_usd: Decimal,
}

/// Compute the published capacities and price without changing the saved plan.
pub fn plan_choice(plan: Plan) -> PlanChoice {
    let (starts, machines, price) = base(plan.tier);
    let add_ons = if plan.tier == Tier::Free {
        AddOns::default()
    } else {
        plan.add_ons
    };
    PlanChoice {
        plan: Plan { add_ons, ..plan },
        starts: Decimal::from(starts) + Decimal::from(add_ons.start_blocks) * Decimal::from(10_000),
        machines: u128::from(machines) + u128::from(add_ons.machines),
        monthly_usd: Decimal::from(price)
            + Decimal::from(add_ons.start_blocks) * Decimal::from(10)
            + Decimal::from(add_ons.machines) * Decimal::from(20),
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
    let (starts, machines, _) = base(tier);
    let blocks = ((expected - Decimal::from(starts)).max(Decimal::ZERO) / Decimal::from(10_000))
        .ceil()
        .to_u64()
        .unwrap_or_default();
    plan_choice(Plan {
        tier,
        add_ons: AddOns {
            start_blocks: blocks,
            machines: machine_count.saturating_sub(machines),
        },
    })
}

fn base(tier: Tier) -> (u64, u64, u64) {
    match tier {
        Tier::Free => (2_000, 1, 0),
        Tier::Team => (25_000, 3, 49),
        Tier::Business => (150_000, 10, 299),
    }
}

#[cfg(test)]
#[path = "_tests_/pricing_tests.rs"]
mod pricing_tests;
