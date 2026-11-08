//! Object-safe billing persistence and exclusive session ownership.

use chrono::{DateTime, Utc};
use unimock::unimock;
use uuid::Uuid;

use crate::billing::commands::{BillingCommand, CheckoutProgress};
use crate::billing::quotes::StoredQuote;
use crate::billing::renewals::Renewal;
use crate::billing::state::BillingState;
use crate::error::Result;

/// A consistent read of confirmed state and unfinished work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingRecord {
    /// The confirmed state used by metering and the dashboard.
    pub state: BillingState,
    /// A durable Stripe mutation awaiting reconciliation.
    pub command: Option<BillingCommand>,
    /// An open or confirming Checkout.
    pub checkout: Option<CheckoutProgress>,
    /// Last-organization removal requested period-end cancellation.
    pub removal_cancel_requested: bool,
}

/// Billing queries shared by PostgreSQL and service test doubles.
#[unimock(api = BillingStoreMock)]
pub trait BillingStore: Send + Sync {
    /// Read billing without creating a Stripe customer or card.
    fn billing(&self, workspace_id: Uuid) -> Result<BillingRecord>;
    /// Resolve the workspace owning a currently retained Stripe subscription.
    fn billing_workspace_for_subscription(&self, subscription_id: &str) -> Result<Option<Uuid>>;
    /// Acquire exclusive billing access on a dedicated live connection.
    ///
    /// Return None when another command or event owns the workspace.
    /// Closing the returned handle releases its database session lock.
    fn try_lock_billing(&self, workspace_id: Uuid) -> Result<Option<Box<dyn BillingSession>>>;
    /// Find durable App-removal requests which still need Stripe work.
    fn billing_cancellations(&self) -> Result<Vec<Uuid>>;
}

/// The exclusive billing session, kept alive across Stripe calls.
#[unimock(api = BillingSessionMock)]
pub trait BillingSession: Send {
    /// Read a quote only within the locked workspace.
    fn quote(&mut self, quote_id: Uuid) -> Result<Option<StoredQuote>>;
    /// Save an authoritative review and prune expired reviews without ongoing work.
    fn save_quote(&mut self, quote: &StoredQuote) -> Result<()>;
    /// Consume one unused quote and persist its command in one transaction.
    fn consume_quote(&mut self, quote_id: Uuid, command: &BillingCommand) -> Result<bool>;
    /// Read the workspace's latest billing state.
    fn record(&mut self) -> Result<BillingRecord>;
    /// Read every retained renewal outcome for one subscription.
    fn renewals(&mut self, subscription_id: &str) -> Result<Vec<Renewal>>;
    /// Persist a verified invoice outcome before any resulting provider mutation.
    fn save_renewal(&mut self, renewal: &Renewal) -> Result<()>;
    /// Persist a mutation before sending it to Stripe.
    fn save_command(&mut self, command: &BillingCommand) -> Result<()>;
    /// Persist the exact Checkout returned by an idempotent mutation.
    fn save_checkout(&mut self, checkout: &CheckoutProgress) -> Result<()>;
    /// Detect an already committed event before locked reconciliation reads.
    fn event_applied(&mut self, event_id: &str) -> Result<bool>;
    /// Commit state, optional event receipt, and command completion atomically.
    ///
    /// None for state preserves the confirmed state. A completed command also
    /// clears Checkout. A handled removal clears only its durable request.
    fn commit(
        &mut self,
        state: Option<&BillingState>,
        event_id: Option<&str>,
        finish_command: bool,
        handled_removal: bool,
        at: DateTime<Utc>,
    ) -> Result<()>;
}
