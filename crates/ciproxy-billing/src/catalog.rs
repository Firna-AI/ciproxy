//! The USD monthly catalog and its user-selected quantities.

use ciproxy_store_interface::billing::state::{Selection, Tier};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// A stable Stripe lookup key belonging to this catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PriceCode {
    /// Team at $49 each month.
    #[serde(rename = "ciproxy_team_monthly_usd")]
    Team,
    /// Business at $299 each month.
    #[serde(rename = "ciproxy_business_monthly_usd")]
    Business,
    /// One block of 10,000 starts at $10 each month.
    #[serde(rename = "ciproxy_starts_monthly_usd")]
    Starts,
    /// One additional machine at $20 each month.
    #[serde(rename = "ciproxy_machine_monthly_usd")]
    Machine,
}

impl PriceCode {
    /// The stable lookup key written to Stripe prices.
    pub fn lookup_key(self) -> &'static str {
        match self {
            Self::Team => "ciproxy_team_monthly_usd",
            Self::Business => "ciproxy_business_monthly_usd",
            Self::Starts => "ciproxy_starts_monthly_usd",
            Self::Machine => "ciproxy_machine_monthly_usd",
        }
    }

    /// The monthly USD amount in cents.
    pub fn cents(self) -> u64 {
        match self {
            Self::Team => 4900,
            Self::Business => 29900,
            Self::Starts => 1000,
            Self::Machine => 2000,
        }
    }
}

/// Monthly total, today's charge, and recurring day in a billing review.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutReview {
    /// The monthly amount in USD cents, with no tax.
    pub monthly_cents: u64,
    /// The amount payable today, in USD cents.
    pub today_cents: u64,
    /// The recurring monthly billing day in UTC.
    pub billing_day: u32,
}

/// Calculate the selected monthly total without choosing a tier from use.
pub fn monthly_total(selection: Selection) -> Result<u64> {
    let base = match selection.tier {
        Tier::Free if selection.add_ons.start_blocks == 0 && selection.add_ons.machines == 0 => 0,
        Tier::Free => return Err(Error::FreeAddOns),
        Tier::Team => PriceCode::Team.cents(),
        Tier::Business => PriceCode::Business.cents(),
    };
    Ok(base
        + u64::from(selection.add_ons.start_blocks) * 1000
        + u64::from(selection.add_ons.machines) * 2000)
}

pub(crate) fn line_items(selection: Selection) -> Result<Vec<(PriceCode, u32)>> {
    let tier = match selection.tier {
        Tier::Team => PriceCode::Team,
        Tier::Business => PriceCode::Business,
        Tier::Free => return Err(Error::PaidTierRequired),
    };
    let mut items = vec![(tier, 1)];
    if selection.add_ons.start_blocks > 0 {
        items.push((PriceCode::Starts, selection.add_ons.start_blocks));
    }
    if selection.add_ons.machines > 0 {
        items.push((PriceCode::Machine, selection.add_ons.machines));
    }
    Ok(items)
}
