//! Persisted command inputs and Checkout progress.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::billing::quotes::QuotePayment;
use crate::billing::state::Selection;

/// An exact retryable Stripe mutation with a server-owned identifier.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BillingCommand {
    /// The identifier used for the Stripe idempotency key.
    pub id: Uuid,
    /// The first attempt time, used for Checkout discovery after lost replies.
    pub started_at: DateTime<Utc>,
    /// Stripe-confirmed deadline of an unapplied charge, if present.
    pub payment_expires_at: Option<DateTime<Utc>>,
    /// The immutable mutation input.
    pub kind: CommandKind,
}

/// The supported Stripe mutations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommandKind {
    /// Create one Checkout for a Free workspace.
    Checkout {
        /// The selected paid tier and quantities.
        selection: Selection,
        /// The trusted dashboard return URL.
        success_url: String,
        /// The trusted dashboard cancel URL.
        cancel_url: String,
    },
    /// Replace paid items with Stripe proration.
    Change {
        /// The chosen paid tier and quantities.
        selection: Selection,
        /// Whether payment must succeed before Stripe applies the change.
        upgrade: bool,
        /// Reviewed inputs for a paid tier change, absent for quantity-only changes.
        #[serde(default)]
        quote: Option<QuotePayment>,
    },
    /// End the paid subscription at its existing period end.
    Cancel,
}

/// One unfinished Checkout, kept until Stripe confirms or expires it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckoutProgress {
    /// The command which created this session.
    pub command_id: Uuid,
    /// The Stripe session identifier.
    pub session_id: String,
    /// The hosted Checkout URL, absent after completion.
    pub url: Option<String>,
}
