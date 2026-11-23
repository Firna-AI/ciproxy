//! Credential-free overflow observations for the live queue.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::money::Cents;
use super::overflow::OverflowProvider;

/// Running jobs counted from their saved placement.
#[derive(Serialize, ToSchema)]
pub struct ProviderRunningCount {
    /// Provider that runs the jobs.
    pub provider: OverflowProvider,
    /// Running job count as a decimal string.
    pub running: String,
}

/// Current overflow observations for one provider.
#[derive(Serialize, ToSchema)]
pub struct JobOverflow {
    /// Provider whose account, refusals, and spend were observed.
    pub provider: OverflowProvider,
    /// An active account is connected.
    pub connected: bool,
    /// The provider rejected the active key.
    pub key_rejected: bool,
    /// The provider requires account billing before it starts more sandboxes.
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
