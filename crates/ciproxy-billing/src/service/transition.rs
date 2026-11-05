//! Pure conversion of current Stripe state into confirmed local coverage.

use chrono::{DateTime, Months, Utc};
use ciproxy_store_interface::billing::renewals::Renewal;
use ciproxy_store_interface::billing::state::{
    AddOns, BillingState, Selection, SubscriptionState, Tier,
};

use crate::catalog::PriceCode;
use crate::error::{Error, Result};
use crate::stripe::wire::invoice::CancelReason;

use crate::stripe::wire::types::{Status, Subscription};

pub(super) fn selection(subscription: &Subscription) -> Result<Selection> {
    if subscription.items.has_more {
        return Err(Error::StripeState);
    }
    let mut tier = None;
    let mut add_ons = AddOns::default();
    let mut seen = Vec::new();
    for item in &subscription.items.data {
        let code = item.price.lookup_key;
        if seen.contains(&code) {
            return Err(Error::StripeState);
        }
        seen.push(code);
        match code {
            PriceCode::Team | PriceCode::Business => {
                if item.quantity != 1 || tier.is_some() {
                    return Err(Error::StripeState);
                }
                tier = Some(if code == PriceCode::Team {
                    Tier::Team
                } else {
                    Tier::Business
                });
            }
            PriceCode::Starts => add_ons.start_blocks = item.quantity,
            PriceCode::Machine => add_ons.machines = item.quantity,
        }
    }
    Ok(Selection {
        tier: tier.ok_or(Error::StripeState)?,
        add_ons,
    })
}

pub(super) fn confirmed(
    old: &BillingState,
    current: &Subscription,
    now: DateTime<Utc>,
    renewal: Option<&Renewal>,
) -> Result<BillingState> {
    if current.metadata.workspace_id != Some(old.workspace_id)
        || old
            .customer_id
            .as_ref()
            .is_some_and(|id| id != &current.customer)
    {
        return Err(Error::Ownership);
    }
    let mut state = old.clone();
    state.customer_id = Some(current.customer.clone());
    state.subscription_id = Some(current.id.clone());
    if terminal(current) {
        if old.subscription_id.as_ref() != Some(&current.id)
            || old.status != SubscriptionState::Ended
        {
            let ended = current.ended_at.map(timestamp).transpose()?.unwrap_or(now);
            state.free_started_at = ended;
            let payment_failed = current
                .cancellation_details
                .as_ref()
                .is_some_and(|details| details.reason == Some(CancelReason::PaymentFailed));
            state.ended_banner_until =
                if payment_failed || old.status == SubscriptionState::PaymentFailed {
                    Some(
                        ended
                            .checked_add_months(Months::new(1))
                            .ok_or(Error::StripeState)?,
                    )
                } else {
                    None
                };
        }
        state.tier = Tier::Free;
        state.add_ons = AddOns::default();
        state.status = SubscriptionState::Ended;
        state.paid_period_start = None;
        state.paid_period_end = None;
        state.cancel_at = None;
        state.failed_at = None;
        state.failed_invoice_id = None;
        state.next_retry_at = None;
        return Ok(state);
    }
    if !matches!(
        current.status,
        Status::Active | Status::PastDue | Status::Unpaid
    ) {
        return Err(Error::StripeState);
    }
    let selected = selection(current)?;
    let base = current.items.data.first().ok_or(Error::StripeState)?;
    if current.items.data.iter().any(|item| {
        item.current_period_start != base.current_period_start
            || item.current_period_end != base.current_period_end
    }) {
        return Err(Error::StripeState);
    }
    state.tier = selected.tier;
    state.add_ons = selected.add_ons;
    state.paid_period_start = Some(timestamp(base.current_period_start)?);
    state.paid_period_end = Some(timestamp(base.current_period_end)?);
    state.cancel_at = current.cancel_at.map(timestamp).transpose()?.or({
        if current.cancel_at_period_end {
            state.paid_period_end
        } else {
            None
        }
    });
    state.ended_banner_until = None;
    if let Some(renewal) = renewal {
        state.status = SubscriptionState::PaymentFailed;
        state.failed_at = renewal.failed_at;
        state.failed_invoice_id = Some(renewal.invoice_id.clone());
        state.next_retry_at = renewal.next_retry_at;
    } else {
        state.status = SubscriptionState::Active;
        state.failed_at = None;
        state.failed_invoice_id = None;
        state.next_retry_at = None;
    }
    Ok(state)
}

fn timestamp(seconds: i64) -> Result<DateTime<Utc>> {
    DateTime::from_timestamp(seconds, 0).ok_or(Error::StripeState)
}

pub(super) fn terminal(current: &Subscription) -> bool {
    matches!(current.status, Status::Canceled | Status::IncompleteExpired)
}
