//! Durable renewal outcomes independent of the current payment banner.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A verified invoice outcome retained after payment or subscription end.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Renewal {
    /// The customer that owns the invoice.
    pub customer_id: String,
    /// The subscription that generated the renewal.
    pub subscription_id: String,
    /// The stable renewal invoice identifier.
    pub invoice_id: String,
    /// The last outcome from a fresh provider read.
    pub outcome: RenewalOutcome,
    /// The outstanding amount in cents, when the provider supplied it.
    pub amount_remaining: Option<i64>,
    /// The most recent signed failed-payment time for this invoice.
    pub failed_at: Option<DateTime<Utc>>,
    /// The next provider retry for this invoice, when available.
    pub next_retry_at: Option<DateTime<Utc>>,
    /// Verified automatic collection stop, retained for cancellation retries.
    pub automatic_stop: Option<AutomaticStop>,
    /// Latest signed collection change, including manual changes that supersede a stop.
    pub collection_changed_at: Option<DateTime<Utc>>,
}

/// Invoice outcomes which determine whether payment remains due.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RenewalOutcome {
    /// The invoice is still being prepared.
    Draft,
    /// An open invoice remains unpaid.
    Open,
    /// Payment completed.
    Paid,
    /// The invoice was voided.
    Void,
    /// Stripe marked the invoice uncollectible.
    Uncollectible,
}

/// Signed evidence verified against current owned subscription and invoice reads.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct AutomaticStop {
    /// The verified event that stopped automatic collection.
    pub event_id: String,
    /// The provider event creation time.
    pub stopped_at: DateTime<Utc>,
    /// The attempt that stopped collection, without a fixed retry limit.
    pub attempt_count: u32,
}
