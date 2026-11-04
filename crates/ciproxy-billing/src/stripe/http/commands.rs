//! Stripe provider calls for paid changes, cancellation, and customer portal.

use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::Selection;
use serde::Deserialize;
use uuid::Uuid;

use crate::catalog::PriceCode;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::api::StripeApi;
use crate::stripe::forms;
use crate::stripe::wire::invoice::RetrievedInvoice;
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

    fn change(
        &self,
        current: &Subscription,
        selection: Selection,
        command: &BillingCommand,
    ) -> Result<Subscription> {
        let CommandKind::Change { upgrade, .. } = command.kind else {
            return Err(Error::StripeState);
        };
        let prices = self.prices(selection)?;
        let form = forms::replacement(current, selection, &prices, upgrade)?;
        let key = format!("ciproxy:change:{}", command.id);
        self.post(
            &format!("/v1/subscriptions/{}", current.id),
            &form,
            Some(&key),
        )?;
        self.subscription(&current.id)
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
