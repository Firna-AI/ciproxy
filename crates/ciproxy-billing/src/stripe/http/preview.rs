//! Invoice previews for the immediate change and its future monthly amount.

use chrono::{DateTime, Datelike, Utc};
use ciproxy_store_interface::billing::quotes::QuotePayment;
use ciproxy_store_interface::billing::state::Selection;
use serde::Deserialize;
use uuid::Uuid;

use crate::catalog::CheckoutReview;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::api::PaidPreview;
use crate::stripe::forms;
use crate::stripe::wire::invoice::InvoiceParent;
use crate::stripe::wire::types::Subscription;

use super::client::HttpStripe;

#[derive(Deserialize)]
struct PreviewInvoice {
    customer: String,
    parent: Option<InvoiceParent>,
    currency: String,
    total: i64,
    amount_due: u64,
}

impl PreviewInvoice {
    fn check(&self, current: &Subscription) -> Result<()> {
        if self.customer != current.customer
            || self
                .parent
                .as_ref()
                .and_then(|parent| parent.subscription_details.as_ref())
                .is_none_or(|details| details.subscription != current.id)
        {
            return Err(Error::Ownership);
        }
        if self.currency != "usd" {
            return Err(Error::StripeState);
        }
        Ok(())
    }
}

impl HttpStripe {
    pub(super) fn preview_change(
        &self,
        current: &Subscription,
        selection: Selection,
        at: DateTime<Utc>,
        upgrade: bool,
    ) -> Result<PaidPreview> {
        let billing_day = anchor_day(current)?;
        let prices = self.prices(selection)?;
        let quote = QuotePayment {
            id: Uuid::nil(),
            proration_at: at,
            items: forms::changes(current, selection, &prices)?,
        };
        let mut form = vec![
            ("customer".into(), current.customer.clone()),
            ("subscription".into(), current.id.clone()),
        ];
        let mut immediate = form.clone();
        immediate.push(("preview_mode".into(), "next".into()));
        immediate.extend(forms::reviewed(&quote, upgrade, true));
        let next = self.preview_invoice(&immediate, current)?;
        form.extend(forms::item_fields(
            &quote.items,
            "subscription_details[items]",
        ));
        form.push(("preview_mode".into(), "recurring".into()));
        let recurring = self.preview_invoice(&form, current)?;
        Ok(PaidPreview {
            amounts: CheckoutReview {
                monthly_cents: recurring.total.try_into().internal()?,
                today_cents: if upgrade { next.amount_due } else { 0 },
                billing_day,
            },
            items: quote.items,
        })
    }

    fn preview_invoice(
        &self,
        form: &[(String, String)],
        current: &Subscription,
    ) -> Result<PreviewInvoice> {
        let invoice = self
            .post("/v1/invoices/create_preview", form, None)?
            .json::<PreviewInvoice>()
            .internal()?;
        invoice.check(current)?;
        Ok(invoice)
    }
}

fn anchor_day(current: &Subscription) -> Result<u32> {
    let day = if let Some(config) = &current.billing_cycle_anchor_config {
        config.day_of_month
    } else {
        current
            .billing_cycle_anchor
            .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0))
            .ok_or(Error::StripeState)?
            .day()
    };
    if !(1..=31).contains(&day) {
        return Err(Error::StripeState);
    }
    Ok(day)
}
