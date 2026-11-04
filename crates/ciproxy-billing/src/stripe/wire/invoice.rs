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
