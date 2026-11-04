//! Stripe provider boundary and its pinned HTTP implementation.

use ciproxy_store_interface::billing::commands::BillingCommand;
use ciproxy_store_interface::billing::state::Selection;
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;

use super::wire::types::{CheckoutSession, Subscription};

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
    /// Read the current invoice parent for verified invoice-event routing.
    fn invoice_subscription(&self, invoice_id: &str) -> Result<Option<String>>;
    /// Replace paid items with proration and payment-gated upgrades.
    fn change(
        &self,
        current: &Subscription,
        selection: Selection,
        command: &BillingCommand,
    ) -> Result<Subscription>;
    /// Schedule period-end cancellation without refunds or period changes.
    fn cancel(&self, subscription_id: &str, command_id: Uuid) -> Result<Subscription>;
    /// Open an explicitly restricted portal configuration for this customer.
    fn portal(&self, customer_id: &str, return_url: &str) -> Result<String>;
}
