//! Credential-free overflow observations for the live queue.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::money::Cents;

/// Existing overflow service observations, absent when that source is unavailable.
#[derive(Serialize, ToSchema)]
pub struct JobOverflow {
    /// An active Boat account is connected.
    pub connected: bool,
    /// Boat rejected the active key.
    pub key_rejected: bool,
    /// Boat requires account billing before it starts more sandboxes.
    pub billing_required: bool,
    /// Provider-supplied next retry date.
    pub retry_at: Option<DateTime<Utc>>,
    /// Saved monthly spend limit in integer US cents.
    pub limit_cents: Option<Cents>,
    /// Conservatively counted spend, absent when safe bounds are unavailable.
    pub counted_cents: Option<Cents>,
    /// Current spend has reached the monthly limit, absent with incomplete bounds.
    pub limit_reached: Option<bool>,
    /// True when all provider start and runtime measurements are complete.
    pub measured_complete: bool,
    /// First UTC instant at which the monthly budget resets.
    pub next_month: DateTime<Utc>,
    /// Provider-refused size names until a catalog refresh.
    pub refused_sizes: Vec<String>,
}
