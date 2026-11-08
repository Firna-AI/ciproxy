//! Object-safe service entry points for dashboard and webhook callers.

use ciproxy_store_interface::billing::state::{AddOns, Selection};
use ciproxy_store_interface::billing::store::BillingRecord;
use unimock::unimock;
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::catalog::CheckoutReview;
use crate::error::Result;

use super::view::{ChangeOutcome, CheckoutOutcome, EventOutcome};

/// Workspace billing. Async hosts call these methods on blocking workers.
#[unimock(api = BillingApiMock)]
pub trait BillingApi: Send + Sync {
    /// Read state and banners for any authenticated workspace member.
    fn status(&self, access: BillingAccess) -> Result<BillingRecord>;
    /// Review a selected Checkout before the payer leaves the dashboard.
    fn review(&self, access: BillingAccess, selection: Selection) -> Result<CheckoutReview>;
    /// Open or recover the workspace's one unfinished Checkout.
    fn open_checkout(&self, access: BillingAccess, selection: Selection)
    -> Result<CheckoutOutcome>;
    /// Reconcile the return page using the same Stripe read as a webhook.
    fn reconcile_checkout(
        &self,
        access: BillingAccess,
        session_id: &str,
    ) -> Result<CheckoutOutcome>;
    /// Start or recover payment for an exact authoritative paid quote.
    fn change(&self, access: BillingAccess, quote_id: Uuid) -> Result<ChangeOutcome>;
    /// Save quantities while keeping the Stripe tier reconciled under the billing lock.
    fn change_add_ons(&self, access: BillingAccess, add_ons: AddOns) -> Result<ChangeOutcome>;
    /// Keep paid coverage until Stripe actually ends the subscription.
    fn move_to_free(&self, access: BillingAccess) -> Result<()>;
    /// Open invoices, saved card, cancellation, and undo-cancellation only.
    fn portal(&self, access: BillingAccess) -> Result<String>;
    /// Verify raw request bytes before parsing or processing a Stripe event.
    fn webhook(&self, signature: &str, raw: &[u8]) -> Result<EventOutcome>;
    /// Retry durable last-organization cancellation requests.
    ///
    /// The host must poll this method and retry errors. Organization removal
    /// writes the request in the same database transaction as the removal.
    fn reconcile_removal(&self, workspace_id: Uuid) -> Result<()>;
    /// Recover owned credit transfers and invoice checks after replies or events are lost.
    fn reconcile_payments(&self, workspace_id: Uuid) -> Result<()>;
}
