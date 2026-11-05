//! Durable cancellation decisions for individual signed collection-stop events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The cancellation history of one renewal invoice.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct CollectionDecisions {
    /// The latest accepted signed collection transition.
    pub changed_at: Option<DateTime<Utc>>,
    /// Decisions retained by event identifier, including terminal decisions.
    pub decisions: Vec<CancellationDecision>,
}

/// A cancellation decision tied permanently to one automatic-stop event.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CancellationDecision {
    /// The signed event and attempt that supplied the cancellation evidence.
    pub stop: AutomaticStop,
    /// Whether that event can still authorize cancellation.
    pub state: CancellationState,
}

/// The durable cancellation authority of an automatic-stop event.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationState {
    /// Fresh provider reads permit cancellation or a retry.
    Eligible,
    /// A later change or fresh mismatch permanently retired this event.
    Superseded,
    /// A matching confirmed Stripe end completed this event's cancellation.
    Completed,
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
