//! Stripe provider calls for paid changes, cancellation, and customer portal.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::Selection;
use serde::Deserialize;
use uuid::Uuid;

use crate::catalog::PriceCode;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::api::{PaidPreview, StripeApi};
use crate::stripe::forms;
use crate::stripe::wire::invoice::RetrievedInvoice;
use crate::stripe::wire::pending::InvoiceItem;
use crate::stripe::wire::types::{CheckoutSession, Subscription};

use super::client::HttpStripe;

#[derive(Deserialize)]
struct Portal {
    url: String,
}

impl StripeApi for HttpStripe {
    fn provision(&self) -> Result<()> {
        for code in [
            PriceCode::Team,
            PriceCode::Business,
            PriceCode::Starts,
            PriceCode::Machine,
        ] {
            self.ensure_price(code)?;
        }
        self.portal_configuration()?;
        Ok(())
    }

    fn checkout(
        &self,
        workspace_id: Uuid,
        customer_id: Option<&str>,
        command: &BillingCommand,
    ) -> Result<CheckoutSession> {
        self.create_checkout(workspace_id, customer_id, command)
    }

    fn checkout_session(&self, session_id: &str) -> Result<CheckoutSession> {
        self.get(&format!("/v1/checkout/sessions/{session_id}"), &[])?
            .json::<CheckoutSession>()
            .internal()
    }

    fn subscription(&self, subscription_id: &str) -> Result<Subscription> {
        let subscription = self
            .get(
                &format!("/v1/subscriptions/{subscription_id}"),
                &[("expand[0]".into(), "latest_invoice".into())],
            )?
            .json::<Subscription>()
            .internal()?;
        if subscription.items.has_more {
            return Err(Error::StripeState);
        }
        Ok(subscription)
    }

    fn invoice(&self, invoice_id: &str) -> Result<RetrievedInvoice> {
        self.get(&format!("/v1/invoices/{invoice_id}"), &[])?
            .json::<RetrievedInvoice>()
            .internal()
    }

    fn pending_items(&self, customer_id: &str) -> Result<Vec<InvoiceItem>> {
        self.list_pending_items(customer_id)
    }

    fn customer_balance(&self, customer_id: &str) -> Result<i64> {
        self.read_customer_balance(customer_id)
    }

    fn invoice_item(&self, item_id: &str) -> Result<Option<InvoiceItem>> {
        self.read_invoice_item(item_id)
    }

    fn delete_invoice_item(&self, item_id: &str) -> Result<()> {
        self.remove_invoice_item(item_id)
    }

    fn mark_payment(&self, subscription_id: &str, quote_id: Uuid, command_id: Uuid) -> Result<()> {
        self.stamp_payment(subscription_id, quote_id, command_id)
    }

    fn payment_invoices(
        &self,
        customer_id: &str,
        subscription_id: &str,
    ) -> Result<Vec<RetrievedInvoice>> {
        self.list_payment_invoices(customer_id, subscription_id)
    }

    fn restore_balance(
        &self,
        customer_id: &str,
        command_id: Uuid,
        amount_cents: i64,
    ) -> Result<String> {
        self.restore_customer_balance(customer_id, command_id, amount_cents)
    }

    fn change(
        &self,
        current: &Subscription,
        selection: Selection,
        command: &BillingCommand,
    ) -> Result<Subscription> {
        let CommandKind::Change {
            upgrade, ref quote, ..
        } = command.kind
        else {
            return Err(Error::StripeState);
        };
        let form = match quote {
            Some(quote) => forms::reviewed(quote, upgrade, false),
            None => forms::replacement(current, selection, &self.prices(selection)?, upgrade)?,
        };
        let key = format!("ciproxy:change:{}", command.id);
        self.post(
            &format!("/v1/subscriptions/{}", current.id),
            &form,
            Some(&key),
        )?;
        self.subscription(&current.id)
    }

    fn preview(
        &self,
        current: &Subscription,
        selection: Selection,
        at: DateTime<Utc>,
        upgrade: bool,
    ) -> Result<PaidPreview> {
        self.preview_change(current, selection, at, upgrade)
    }

    fn cancel(&self, subscription_id: &str, command_id: Uuid) -> Result<Subscription> {
        let key = format!("ciproxy:cancel:{command_id}");
        self.post(
            &format!("/v1/subscriptions/{subscription_id}"),
            &[("cancel_at_period_end".into(), "true".into())],
            Some(&key),
        )?;
        self.subscription(subscription_id)
    }

    fn cancel_now(&self, subscription_id: &str, invoice_id: &str) -> Result<Subscription> {
        self.delete(
            &format!("/v1/subscriptions/{subscription_id}"),
            &[
                ("invoice_now".into(), "false".into()),
                ("prorate".into(), "false".into()),
            ],
            Some(&format!("ciproxy:renewal-stop:{invoice_id}")),
        )?;
        self.subscription(subscription_id)
    }

    fn portal(&self, customer_id: &str, return_url: &str) -> Result<String> {
        let configuration = self.portal_configuration()?;
        Ok(self
            .post(
                "/v1/billing_portal/sessions",
                &[
                    ("customer".into(), customer_id.into()),
                    ("configuration".into(), configuration),
                    ("return_url".into(), return_url.into()),
                ],
                None,
            )?
            .json::<Portal>()
            .internal()?
            .url)
    }
}
