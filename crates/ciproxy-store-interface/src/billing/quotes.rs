//! Authoritative paid reviews and their immutable Stripe mutation inputs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::billing::state::Selection;
use crate::metering::history::Period;

/// One exact Stripe subscription item replacement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChangeItem {
    /// Remove the existing item.
    Remove {
        /// Stripe item identifier.
        id: String,
    },
    /// Set an existing or newly added item.
    Set {
        /// Existing Stripe item identifier, absent for an addition.
        id: Option<String>,
        /// Exact catalog price selected during review.
        price_id: String,
        /// Reviewed quantity.
        quantity: u32,
    },
}

/// Provider source facts that must still match when payment starts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuoteSource {
    /// Owned Stripe subscription identifier.
    pub subscription_id: String,
    /// Owned Stripe customer identifier.
    pub customer_id: String,
    /// Current confirmed tier and quantities.
    pub selection: Selection,
    /// Current Stripe period.
    pub period: Period,
    /// Existing items, sorted by identifier.
    pub items: Vec<ChangeItem>,
    /// Original billing anchor.
    pub anchor: Option<i64>,
    /// Explicit recurring anchor day.
    pub anchor_day: Option<u32>,
    /// Scheduled period-end cancellation observed before this intent.
    #[serde(default)]
    pub cancel_at_period_end: bool,
    /// Explicit cancellation time observed before this intent.
    #[serde(default)]
    pub cancel_at: Option<i64>,
}

/// An immutable reviewed mutation copied into its durable payment command.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuotePayment {
    /// Authoritative quote identifier.
    pub id: Uuid,
    /// Exact second used by the invoice preview and every mutation retry.
    pub proration_at: DateTime<Utc>,
    /// Exact reviewed item replacements.
    pub items: Vec<ChangeItem>,
    /// Earlier pending items transferred into this payment, if present.
    #[serde(default)]
    pub offset: Option<PendingOffset>,
    /// USD customer balance counted by the immediate preview.
    #[serde(default)]
    pub starting_balance_cents: Option<i64>,
}

/// One earlier invoice item included in the authoritative preview.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingSource {
    /// Stripe invoice item identifier.
    pub id: String,
    /// Owning Stripe customer.
    pub customer_id: String,
    /// Associated subscription, absent for an unscoped customer item.
    pub subscription_id: Option<String>,
    /// Original signed integer US cents before invoice discounts.
    pub amount_cents: i64,
    /// Signed contribution counted in the preview after line discounts.
    pub effective_cents: i64,
}

/// One signed invoice effect carried by the paid mutation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingOffset {
    /// Earlier source items which cannot also reach a later invoice.
    pub sources: Vec<PendingSource>,
    /// Combined signed integer US cents sent to Stripe.
    pub amount_cents: i64,
    /// Existing catalog product used by the one-time inline price.
    pub product_id: String,
}

/// One workspace-owned authoritative paid quote.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoredQuote {
    /// Immutable quote identity, date, and provider items.
    pub payment: QuotePayment,
    /// Authoritative target tier and add-ons.
    pub selection: Selection,
    /// Whether Stripe must invoice before it applies the change.
    pub upgrade: bool,
    /// Provider snapshot used for the review.
    pub source: QuoteSource,
    /// Reviewed monthly integer US cents.
    pub monthly_cents: u64,
    /// Reviewed charge today in integer US cents.
    pub today_cents: u64,
    /// Reviewed recurring UTC billing day.
    pub billing_day: u32,
    /// Exclusive deadline for starting the payment.
    pub expires_at: DateTime<Utc>,
    /// Command which consumed the quote, retained after command completion.
    pub command_id: Option<Uuid>,
}
