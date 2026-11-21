//! Freeze only earlier pending items actually counted by the complete preview.

use std::collections::BTreeMap;

use ciproxy_store_interface::billing::quotes::{PendingOffset, PendingSource};
use serde::Deserialize;

use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::pending::{InvoiceItem, PreviewLine};
use crate::stripe::wire::types::{List, Subscription};

use super::client::HttpStripe;

#[derive(Deserialize)]
struct ProductPrice {
    product: String,
}

impl HttpStripe {
    pub(super) fn preview_offset(
        &self,
        current: &Subscription,
        price: &str,
        before: &[InvoiceItem],
        preview_id: Option<&str>,
        first: Option<List<PreviewLine>>,
    ) -> Result<Option<PendingOffset>> {
        let after = self.list_pending_items(&current.customer)?;
        if before.is_empty() && after.is_empty() {
            return Ok(None);
        }
        let mut page = first.ok_or(Error::StripeState)?;
        let mut lines = Vec::new();
        loop {
            let cursor = page.data.last().map(|line| line.id.clone());
            if page.has_more && cursor.is_none() {
                return Err(Error::StripeState);
            }
            lines.extend(page.data);
            if !page.has_more {
                break;
            }
            page = self
                .get(
                    &format!(
                        "/v1/invoices/{}/lines",
                        preview_id.ok_or(Error::StripeState)?
                    ),
                    &[
                        ("limit".into(), "100".into()),
                        ("starting_after".into(), cursor.ok_or(Error::StripeState)?),
                    ],
                )?
                .json::<List<PreviewLine>>()
                .internal()?;
        }
        let mut amounts = BTreeMap::<String, (i128, i128)>::new();
        for line in lines {
            let Some(id) = line.pending_id() else {
                continue;
            };
            if !before.iter().any(|item| item.id == id) && !after.iter().any(|item| item.id == id) {
                continue;
            }
            if line.currency != "usd" {
                return Err(Error::StripeState);
            }
            let discount: i128 = line
                .discount_amounts
                .iter()
                .map(|entry| i128::from(entry.amount))
                .sum();
            let entry = amounts.entry(id.into()).or_default();
            entry.0 += i128::from(line.amount);
            entry.1 += i128::from(line.amount) - discount;
        }
        let mut sources = Vec::new();
        let mut total = 0_i128;
        for (id, (raw, effective)) in amounts {
            let item = after
                .iter()
                .find(|item| item.id == id)
                .ok_or(Error::StripeState)?;
            if item.currency != "usd"
                || item.customer != current.customer
                || item.subscription_id().is_some_and(|id| id != current.id)
                || i128::from(item.amount) != raw
            {
                return Err(Error::StripeState);
            }
            sources.push(PendingSource {
                id,
                customer_id: item.customer.clone(),
                subscription_id: item.subscription_id().map(str::to_owned),
                amount_cents: item.amount,
                effective_cents: effective.try_into().internal()?,
            });
            total += effective;
        }
        if sources.is_empty() {
            return Ok(None);
        }
        let product = self
            .get(&format!("/v1/prices/{price}"), &[])?
            .json::<ProductPrice>()
            .internal()?;
        Ok(Some(PendingOffset {
            sources,
            amount_cents: total.try_into().internal()?,
            product_id: product.product,
        }))
    }
}
