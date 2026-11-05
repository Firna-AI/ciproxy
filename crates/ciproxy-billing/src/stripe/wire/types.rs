//! Typed Stripe wire objects for API version 2026-10-28.endive.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::invoice::{CancellationDetails, Invoice};
use crate::catalog::PriceCode;

/// Server-owned metadata attached to Stripe sessions and subscriptions.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Metadata {
    /// The workspace owning the Stripe object.
    pub workspace_id: Option<Uuid>,
    /// The durable command that created the object.
    pub command_id: Option<Uuid>,
}

/// Current hosted Checkout state.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CheckoutSession {
    /// Stripe's session identifier.
    pub id: String,
    /// The current session state.
    pub status: CheckoutStatus,
    /// Whether the initial charge is confirmed.
    pub payment_status: PaymentStatus,
    /// The current hosted redirect URL.
    pub url: Option<String>,
    /// The created Stripe customer.
    pub customer: Option<String>,
    /// The created subscription, retrieved separately after confirmation.
    pub subscription: Option<String>,
    /// Server-owned ownership metadata.
    pub metadata: Metadata,
}

/// Checkout lifecycle states.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckoutStatus {
    /// The customer can continue Checkout.
    Open,
    /// Checkout finished, possibly awaiting charge confirmation.
    Complete,
    /// Checkout ended without completion.
    Expired,
}

/// Checkout charge confirmation states.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    /// Payment succeeded.
    Paid,
    /// Stripe has not confirmed payment.
    Unpaid,
    /// No money was due.
    NoPaymentRequired,
}

/// Stripe list envelope.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct List<T> {
    /// The current page of items.
    pub data: Vec<T>,
    /// More items exist beyond this page.
    #[serde(default)]
    pub has_more: bool,
}

/// A catalog price returned by Stripe.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Price {
    /// Stripe price identifier.
    pub id: String,
    /// The known catalog lookup key.
    pub lookup_key: PriceCode,
}

/// One confirmed subscription item, with item-level paid-period bounds.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    /// Stripe item identifier for replacement or removal.
    pub id: String,
    /// The expanded price.
    pub price: Price,
    /// The billed monthly quantity.
    pub quantity: u32,
    /// Inclusive period start in Unix seconds.
    pub current_period_start: i64,
    /// Exclusive period end in Unix seconds.
    pub current_period_end: i64,
}

/// Current subscription, read rather than trusting event snapshots.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Subscription {
    /// Stripe's subscription identifier.
    pub id: String,
    /// Stripe's customer identifier.
    pub customer: String,
    /// Server-owned ownership metadata.
    pub metadata: Metadata,
    /// Stripe's current subscription state.
    pub status: Status,
    /// The complete current item list.
    pub items: List<Item>,
    /// An unapplied item change waiting for a successful charge.
    pub pending_update: Option<PendingUpdate>,
    /// The current expanded invoice.
    pub latest_invoice: Option<Invoice>,
    /// Whether the subscription ends at its current period end.
    pub cancel_at_period_end: bool,
    /// Stripe's explicit scheduled end, if set.
    pub cancel_at: Option<i64>,
    /// The actual end, when the subscription has ended.
    pub ended_at: Option<i64>,
    /// Why Stripe ended the subscription.
    pub cancellation_details: Option<CancellationDetails>,
}

/// Stripe subscription states relevant to paid coverage.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Paid coverage continues.
    Active,
    /// A failed renewal is being retried.
    PastDue,
    /// Stripe stopped retries but the subscription has not ended.
    Unpaid,
    /// A subscription ended.
    Canceled,
    /// An initial payment is incomplete.
    Incomplete,
    /// An initial payment expired.
    IncompleteExpired,
    /// An external trial, not offered by this catalog.
    Trialing,
    /// An unsupported paused state.
    Paused,
}

/// An unapplied Stripe change and its expiration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PendingUpdate {
    /// When Stripe discards an unpaid pending change.
    pub expires_at: i64,
}
