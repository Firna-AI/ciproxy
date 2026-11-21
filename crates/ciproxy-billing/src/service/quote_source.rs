//! Stable provider facts which fence a review from later subscription changes.

use ciproxy_store_interface::billing::quotes::{ChangeItem, QuoteSource};

use crate::error::Result;
use crate::stripe::wire::types::Subscription;

use super::transition::{period, selection};

pub(super) fn source(current: &Subscription) -> Result<QuoteSource> {
    let mut items = current
        .items
        .data
        .iter()
        .map(|item| ChangeItem::Set {
            id: Some(item.id.clone()),
            price_id: item.price.id.clone(),
            quantity: item.quantity,
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        let id = |item: &ChangeItem| match item {
            ChangeItem::Set { id, .. } => id.clone(),
            ChangeItem::Remove { id } => Some(id.clone()),
        };
        id(left).cmp(&id(right))
    });
    Ok(QuoteSource {
        subscription_id: current.id.clone(),
        customer_id: current.customer.clone(),
        selection: selection(current)?,
        period: period(current)?,
        items,
        anchor: current.billing_cycle_anchor,
        cancel_at_period_end: current.cancel_at_period_end,
        cancel_at: current.cancel_at,
        anchor_day: current
            .billing_cycle_anchor_config
            .as_ref()
            .map(|anchor| anchor.day_of_month),
    })
}
