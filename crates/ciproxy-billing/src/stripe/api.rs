//! Stripe provider boundary and its pinned HTTP implementation.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::commands::BillingCommand;
use ciproxy_store_interface::billing::quotes::ChangeItem;
use ciproxy_store_interface::billing::state::Selection;
use unimock::unimock;
use uuid::Uuid;

use crate::catalog::CheckoutReview;
use crate::error::Result;

use super::wire::invoice::RetrievedInvoice;
use super::wire::types::{CheckoutSession, Subscription};

/// Authoritative amounts and the exact provider items used by both previews.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaidPreview {
    /// Reviewed immediate and recurring amounts.
    pub amounts: CheckoutReview,
    /// Exact item replacements, including resolved catalog price identities.
    pub items: Vec<ChangeItem>,
}

/// The provider operations used by billing orchestration.
#[unimock(api = StripeApiMock)]
pub trait StripeApi: Send + Sync {
    /// Ensure the four monthly USD prices and restricted portal exist.
    fn provision(&self) -> Result<()>;
    /// Find or create the exact Checkout using durable command metadata.
    fn checkout(
        &self,
        workspace_id: Uuid,
        customer_id: Option<&str>,
        command: &BillingCommand,
    ) -> Result<CheckoutSession>;
    /// Retrieve current Checkout state for either the return page or an event.
    fn checkout_session(&self, session_id: &str) -> Result<CheckoutSession>;
    /// Retrieve current subscription with items and latest invoice expanded.
    fn subscription(&self, subscription_id: &str) -> Result<Subscription>;
    /// Retrieve a current invoice and its customer and subscription ownership.
    fn invoice(&self, invoice_id: &str) -> Result<RetrievedInvoice>;
    /// Preview the exact item replacement without changing the subscription.
    fn preview(
        &self,
        current: &Subscription,
        selection: Selection,
        at: DateTime<Utc>,
        upgrade: bool,
    ) -> Result<PaidPreview>;
    /// Replace paid items with proration and payment-gated upgrades.
    fn change(
        &self,
        current: &Subscription,
        selection: Selection,
        command: &BillingCommand,
    ) -> Result<Subscription>;
    /// Schedule period-end cancellation without refunds or period changes.
    fn cancel(&self, subscription_id: &str, command_id: Uuid) -> Result<Subscription>;
    /// End a subscription after verified final renewal failure, without a new charge.
    fn cancel_now(&self, subscription_id: &str, invoice_id: &str) -> Result<Subscription>;
    /// Open an explicitly restricted portal configuration for this customer.
    fn portal(&self, customer_id: &str, return_url: &str) -> Result<String>;
}
