//! Durable pending-item ownership and diagnostic paid-invoice comparisons.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::billing::quotes::StoredQuote;

/// Recovery state which survives quote pruning and command replacement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaymentWork {
    /// Original durable command identifier.
    pub id: Uuid,
    /// Owning customer workspace.
    pub workspace_id: Uuid,
    /// Immutable authoritative review and source facts.
    pub quote: StoredQuote,
    /// First command time for provider discovery.
    pub started_at: DateTime<Utc>,
    /// Whether the subscription's invoice-correlation metadata is confirmed.
    pub marked: bool,
    /// Source whose deletion intent persisted before the provider call.
    pub retiring: Option<String>,
    /// Source items confirmed removed before the paid mutation.
    pub retired: Vec<String>,
    /// First time the main update could have reached Stripe.
    pub submitted_at: Option<DateTime<Utc>>,
    /// Exact update invoice, discovered independently of the latest invoice.
    pub invoice_id: Option<String>,
    /// Financial transfer outcome.
    pub settlement: Settlement,
    /// One optional signed value restoration and its provider identity.
    pub restoration: Option<Restoration>,
    /// Result of comparing this payment's invoice with its review.
    pub audit: InvoiceAudit,
    /// Recovery or operator emission remains necessary.
    pub pending: bool,
    /// An unresolved-recovery operator event has already been raised.
    pub recovery_alerted: bool,
    /// Typed provider validation proof recorded before any uncharged restoration.
    pub rejection: Option<ValidationRejection>,
    /// A cancellation or changed source permanently prevents a fresh priced replay.
    #[serde(default)]
    pub superseded: bool,
}

/// A parameter validation rejection which happened before endpoint execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidationRejection {
    /// Provider HTTP status without a response body.
    pub status: u16,
    /// Narrow documented parameter failure.
    pub reason: ValidationFailure,
}

/// Known parameter validation failures, excluding idempotency and card errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ValidationFailure {
    /// A required parameter is absent.
    MissingParameter,
    /// A submitted parameter is unsupported.
    UnknownParameter,
    /// A referenced provider resource does not exist.
    MissingResource,
}

/// Ownership of the earlier signed value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Settlement {
    /// Prepare metadata and retire original pending items.
    Preparing,
    /// A submitted invoice can still complete.
    Pending,
    /// The exact paid invoice consumed the transferred value.
    Consumed,
    /// Definitive unpaid proof permits restoring removed value.
    Restoring,
    /// Stripe confirmed the one replacement balance adjustment.
    Restored,
    /// No source value moved and no payment remains.
    Abandoned,
    /// Provider evidence cannot yet prove a paid or unpaid outcome.
    Unresolved,
}

/// Immutable restoration inputs and its recovered Stripe receipt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Restoration {
    /// Signed integer US cents owed after an uncharged move.
    pub amount_cents: i64,
    /// Durable operation time before the first restoration attempt.
    pub started_at: DateTime<Utc>,
    /// Confirmed customer-balance transaction, also found after lost replies.
    pub transaction_id: Option<String>,
}

/// Diagnostic result which never changes paid entitlement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum InvoiceAudit {
    /// No final paid invoice is available yet.
    Waiting,
    /// This operation created no immediate paid invoice.
    NoInvoice,
    /// The exact paid invoice matches the reviewed amount.
    Matched {
        /// Final integer US cents due on the invoice.
        amount_cents: u64,
    },
    /// The paid invoice differs and needs an operator event.
    Mismatch {
        /// Final integer US cents due on the invoice.
        amount_cents: u64,
        /// The safe hosted operator event has been raised.
        alerted: bool,
    },
}
