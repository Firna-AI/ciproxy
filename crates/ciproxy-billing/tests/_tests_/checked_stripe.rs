//! Provider responses cannot hide a recorded fake request mismatch.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_billing::error::Result;
use ciproxy_billing::stripe::api::{PaidPreview, StripeApi};
use ciproxy_billing::stripe::wire::invoice::RetrievedInvoice;
use ciproxy_billing::stripe::wire::types::{CheckoutSession, Subscription};
use ciproxy_store_interface::billing::commands::BillingCommand;
use ciproxy_store_interface::billing::state::Selection;
use uuid::Uuid;

use super::fake::Probe;

pub(super) struct CheckedStripe {
    pub(super) provider: Arc<dyn StripeApi>,
    pub(super) probe: Probe,
}

impl StripeApi for CheckedStripe {
    fn provision(&self) -> Result<()> {
        let result = self.provider.provision();
        self.probe.check();
        result
    }

    fn checkout(
        &self,
        workspace_id: Uuid,
        customer_id: Option<&str>,
        command: &BillingCommand,
    ) -> Result<CheckoutSession> {
        let result = self.provider.checkout(workspace_id, customer_id, command);
        self.probe.check();
        result
    }

    fn checkout_session(&self, session_id: &str) -> Result<CheckoutSession> {
        let result = self.provider.checkout_session(session_id);
        self.probe.check();
        result
    }

    fn subscription(&self, subscription_id: &str) -> Result<Subscription> {
        let result = self.provider.subscription(subscription_id);
        self.probe.check();
        result
    }

    fn invoice(&self, invoice_id: &str) -> Result<RetrievedInvoice> {
        let result = self.provider.invoice(invoice_id);
        self.probe.check();
        result
    }

    fn change(
        &self,
        current: &Subscription,
        selection: Selection,
        command: &BillingCommand,
    ) -> Result<Subscription> {
        let result = self.provider.change(current, selection, command);
        self.probe.check();
        result
    }

    fn preview(
        &self,
        current: &Subscription,
        selection: Selection,
        at: DateTime<Utc>,
        upgrade: bool,
    ) -> Result<PaidPreview> {
        let result = self.provider.preview(current, selection, at, upgrade);
        self.probe.check();
        result
    }

    fn cancel(&self, subscription_id: &str, command_id: Uuid) -> Result<Subscription> {
        let result = self.provider.cancel(subscription_id, command_id);
        self.probe.check();
        result
    }

    fn cancel_now(&self, subscription_id: &str, invoice_id: &str) -> Result<Subscription> {
        let result = self.provider.cancel_now(subscription_id, invoice_id);
        self.probe.check();
        result
    }

    fn portal(&self, customer_id: &str, return_url: &str) -> Result<String> {
        let result = self.provider.portal(customer_id, return_url);
        self.probe.check();
        result
    }
}
