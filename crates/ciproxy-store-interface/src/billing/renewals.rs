//! Durable renewal outcomes independent of the current payment banner.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::collection::CollectionDecisions;

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
    /// Whether Stripe attempted payment, absent in older provider projections.
    pub attempted: Option<bool>,
    /// The outstanding amount in cents, when the provider supplied it.
    pub amount_remaining: Option<i64>,
    /// The most recent signed failed-payment time for this invoice.
    pub failed_at: Option<DateTime<Utc>>,
    /// The next provider retry for this invoice, when available.
    pub next_retry_at: Option<DateTime<Utc>>,
    /// Per-event cancellation decisions, retained through retries and recovery.
    pub collection: CollectionDecisions,
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
