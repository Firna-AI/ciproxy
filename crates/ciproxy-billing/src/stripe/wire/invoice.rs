//! Current Stripe invoice and cancellation states.

use serde::{Deserialize, Serialize};

/// Current invoice information used for payment and retry banners.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Invoice {
    /// The current invoice identifier.
    pub id: String,
    /// The invoice creation date in Unix seconds.
    pub created: i64,
    /// Whether Stripe confirmed the invoice payment.
    pub status: InvoiceStatus,
    /// Why this invoice exists.
    pub billing_reason: BillingReason,
    /// Stripe's next scheduled attempt.
    pub next_payment_attempt: Option<i64>,
    /// A Stripe-hosted payment page, including bank confirmation.
    pub hosted_invoice_url: Option<String>,
    /// Remaining amount in cents, absent in older stored provider projections.
    pub amount_remaining: Option<i64>,
    /// Whether Stripe still collects this invoice automatically.
    pub auto_advance: Option<bool>,
    /// Whether Stripe attempted payment.
    pub attempted: Option<bool>,
    /// The provider attempt sequence, independent of its configured retry limit.
    pub attempt_count: Option<u32>,
    /// The collection flow used by this invoice.
    pub collection_method: Option<CollectionMethod>,
}

/// Supported Stripe collection flows.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CollectionMethod {
    /// Stripe automatically charges the saved payment method.
    ChargeAutomatically,
    /// The customer pays an invoice manually.
    SendInvoice,
}

/// Current invoice state.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    /// The invoice is still being prepared.
    Draft,
    /// Payment is still due.
    Open,
    /// Payment completed.
    Paid,
    /// The invoice was voided.
    Void,
    /// The invoice is uncollectible.
    Uncollectible,
}

/// The cause used to distinguish renewals from unapplied upgrades.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BillingReason {
    /// A regular subscription renewal.
    SubscriptionCycle,
    /// A paid tier or quantity change.
    SubscriptionUpdate,
    /// An initial Checkout invoice.
    SubscriptionCreate,
    /// An unrelated invoice cause.
    #[serde(other)]
    Other,
}

/// Stripe's stated cancellation reason.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CancellationDetails {
    /// The cancellation reason, when supplied.
    pub reason: Option<CancelReason>,
}

/// Causes that affect the ended-tier banner.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CancelReason {
    /// Stripe ended the subscription after failed collection.
    PaymentFailed,
    /// Another cancellation reason.
    #[serde(other)]
    Other,
}

/// A retrieved invoice with the ownership needed for renewal reconciliation.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RetrievedInvoice {
    /// The current invoice state, independent of subscription.latest_invoice.
    #[serde(flatten)]
    pub invoice: Invoice,
    /// The customer that owes the invoice.
    pub customer: String,
    /// The subscription parent, absent for unrelated invoices.
    pub parent: Option<InvoiceParent>,
}

/// The typed parent of a subscription invoice.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvoiceParent {
    /// Subscription ownership, absent for another invoice parent kind.
    pub subscription_details: Option<SubscriptionDetails>,
}

/// The subscription that generated this invoice.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionDetails {
    /// The stable Stripe subscription identifier.
    pub subscription: String,
}

impl RetrievedInvoice {
    /// Return the subscription identifier without losing typed invoice state.
    pub fn subscription_id(&self) -> Option<&str> {
        self.parent
            .as_ref()?
            .subscription_details
            .as_ref()
            .map(|details| details.subscription.as_str())
    }
}
