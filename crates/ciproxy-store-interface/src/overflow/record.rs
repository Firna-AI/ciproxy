//! Immutable start identities and durable create, launch, and delete phases.

use std::fmt;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::overflow::account::SizeOffer;

/// Ciphertext for a runner registration and its scoped callback token.
#[derive(Clone, Eq, PartialEq)]
pub struct EncryptedLaunch(pub Vec<u8>);

impl fmt::Debug for EncryptedLaunch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EncryptedLaunch([redacted])")
    }
}

/// The external work that is safe for a resumed delivery.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StartPhase {
    /// Dispatch may have occurred. Only recover reads may follow a restart.
    Creating,
    /// The sandbox exists and needs its private runner input.
    Preparing,
    /// A file write may be in flight. Restart recovery must delete before another write.
    Writing,
    /// A persistent per-sandbox launch marker prevents a second runner.
    Launching,
    /// The guest launch was accepted.
    Running,
    /// Deletion must be retried until the provider confirms completion.
    Deleting,
    /// A definitive refusal proved that no sandbox started.
    Refused,
    /// The sandbox is confirmed absent and billing has ended.
    Ended,
}

/// Credential-free immutable request data and its mutable delivery phase.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StartRecord {
    /// Stable idempotent delivery identifier supplied by the control plane.
    pub delivery_id: Uuid,
    /// Stable runner and consumer sandbox identifier.
    pub runner_id: Uuid,
    /// Organization whose scale set owns the registration.
    pub organization_id: Uuid,
    /// Workspace label identifier.
    pub label_id: Uuid,
    /// Internal scale-set identifier.
    pub scale_set_id: Uuid,
    /// Original requested label dimensions, in decimal GB.
    pub requested_vcpus: u32,
    /// Original requested memory.
    pub requested_memory_gb: u32,
    /// Original requested disk.
    pub requested_disk_gb: u32,
    /// Exact accepted provider size and runtime price snapshot.
    pub size: SizeOffer,
    /// Retained key version used for this delivery.
    pub key_id: Uuid,
    /// Provider correlation mark, scoped to deployment and workspace.
    pub deployment: String,
    /// Exact shared saved-snapshot name or share code.
    pub snapshot: String,
    /// Original one-shot lifetime, never extended by recovery.
    pub lifetime_seconds: u32,
    /// Durable phase written before each external mutation.
    pub phase: StartPhase,
    /// Provider sandbox identity once recovered or accepted.
    pub sandbox_id: Option<String>,
    /// Time before create dispatch, used for conservative live runtime pricing.
    pub starts_at: DateTime<Utc>,
    /// Confirmed billing end, absent while delivery or cleanup remains live.
    pub ends_at: Option<DateTime<Utc>>,
    /// Quoted start charge. Absence preserves an unknown provider charge.
    pub start_usd: Option<Decimal>,
    /// First accepted guest end report, absent before token consumption.
    pub reported_at: Option<DateTime<Utc>>,
    /// Exact per-calendar-month runtime observations from the provider.
    pub runtime: Vec<RuntimeCharge>,
    /// A confirmed delete can leave an unmeasured interval after its last read.
    pub runtime_complete: bool,
}

/// Provider-measured runtime cost for one UTC calendar month.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RuntimeCharge {
    /// The first day of the UTC calendar month.
    pub month: NaiveDate,
    /// Runtime dollars for this sandbox only, excluding start charges.
    pub usd: Decimal,
    /// Provider observation boundary.
    pub until: DateTime<Utc>,
    /// The provider still billed runtime at this observation.
    pub running: bool,
}
