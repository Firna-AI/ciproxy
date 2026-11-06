//! Typed billing conversions and presentation-only money rounding.

use ciproxy_billing::access::{BillingAccess, Role};
use ciproxy_billing::service::view::{ChangeOutcome, CheckoutOutcome};
use ciproxy_metering::pricing;
use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use rust_decimal::{Decimal, RoundingStrategy};
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::wire::billing::{PaymentOutcome, PlanAddOns, PlanSelection, PlanTier};
use crate::wire::money::Cents;

/// Match the existing mockup formatter after all exact metering calculations.
pub(super) fn cents(usd: Decimal) -> Cents {
    let rounded = usd.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    Cents(rounded.mantissa() * 10_i128.pow(2 - rounded.scale()))
}

pub(super) fn tier(tier: Tier) -> PlanTier {
    match tier {
        Tier::Free => PlanTier::Free,
        Tier::Team => PlanTier::Team,
        Tier::Business => PlanTier::Business,
    }
}

pub(super) fn stored_tier(tier: PlanTier) -> Tier {
    match tier {
        PlanTier::Free => Tier::Free,
        PlanTier::Team => Tier::Team,
        PlanTier::Business => Tier::Business,
    }
}

pub(super) fn selection(tier_value: Tier, add_ons: AddOns) -> PlanSelection {
    PlanSelection {
        tier: tier(tier_value),
        add_ons: PlanAddOns {
            start_blocks: add_ons.start_blocks,
            machines: add_ons.machines,
        },
    }
}

pub(super) fn stored(value: PlanSelection) -> Selection {
    Selection {
        tier: stored_tier(value.tier),
        add_ons: AddOns {
            start_blocks: value.add_ons.start_blocks,
            machines: value.add_ons.machines,
        },
    }
}

pub(super) fn plan(tier: Tier, add_ons: AddOns) -> pricing::Plan {
    pricing::Plan {
        tier: match tier {
            Tier::Free => pricing::Tier::Free,
            Tier::Team => pricing::Tier::Team,
            Tier::Business => pricing::Tier::Business,
        },
        add_ons: pricing::AddOns {
            start_blocks: u64::from(add_ons.start_blocks),
            machines: u64::from(add_ons.machines),
        },
    }
}

pub(super) fn choice_selection(value: pricing::Plan) -> Result<PlanSelection> {
    Ok(PlanSelection {
        tier: match value.tier {
            pricing::Tier::Free => PlanTier::Free,
            pricing::Tier::Team => PlanTier::Team,
            pricing::Tier::Business => PlanTier::Business,
        },
        add_ons: PlanAddOns {
            start_blocks: value.add_ons.start_blocks.try_into().internal()?,
            machines: value.add_ons.machines.try_into().internal()?,
        },
    })
}

pub(super) fn access(
    workspace_id: Uuid,
    role: ciproxy_store_interface::dashboard::records::Role,
) -> BillingAccess {
    BillingAccess {
        workspace_id,
        role: match role {
            ciproxy_store_interface::dashboard::records::Role::Admin => Role::Admin,
            ciproxy_store_interface::dashboard::records::Role::Billing => Role::Billing,
            ciproxy_store_interface::dashboard::records::Role::Viewer => Role::Viewer,
        },
    }
}

pub(super) fn checkout(outcome: CheckoutOutcome) -> PaymentOutcome {
    match outcome {
        CheckoutOutcome::Open { url } => PaymentOutcome::Open { url },
        CheckoutOutcome::Confirming => PaymentOutcome::Confirming,
        CheckoutOutcome::Confirmed(_) => PaymentOutcome::Confirmed,
        CheckoutOutcome::Unfinished => PaymentOutcome::Unfinished,
    }
}

pub(super) fn change(outcome: ChangeOutcome) -> PaymentOutcome {
    match outcome {
        ChangeOutcome::Applied(_) => PaymentOutcome::Confirmed,
        ChangeOutcome::PaymentRequired { payment_url } => {
            PaymentOutcome::PaymentRequired { payment_url }
        }
    }
}
