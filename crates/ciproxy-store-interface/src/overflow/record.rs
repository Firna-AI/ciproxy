//! Immutable start identities and durable create, launch, and delete phases.

use std::fmt;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::overflow::account::SizeOffer;
use crate::overflow::billing::MeasuredBillingInterval;
use crate::overflow::known::KnownSandbox;

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
    /// No create request has been sent. Preparation may safely retry this delivery.
    Unsent,
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
    /// Startup authority was withdrawn before any create request was sent.
    CancelledUnsent,
    /// The sandbox is confirmed absent and billing has ended.
    Ended,
}

/// Provider parameters fixed only when dispatch commits, including their exact price.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderParameters {
    /// Exact selected provider size and runtime price snapshot.
    pub size: SizeOffer,
    /// One-shot lifetime, never extended by recovery.
    pub lifetime_seconds: u32,
}

/// Immutable pending demand, optional fixed provider parameters, and observed progress.
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
    /// Own-machine fit known by the scaler when it admitted this Boat demand.
    /// Overflow does not refresh own-machine capacity while replanning the Boat account.
    #[serde(default)]
    pub own_machine_fits: bool,
    /// Absent while unsent. Dispatch fixes these parameters before the first create.
    #[serde(flatten)]
    pub dispatch: Option<ProviderParameters>,
    /// Retained key version used for this delivery.
    pub key_id: Uuid,
    /// Provider correlation mark, scoped to deployment and workspace.
    pub deployment: String,
    /// Exact shared saved-snapshot name or share code.
    pub snapshot: String,
    /// Durable phase written before each external mutation.
    pub phase: StartPhase,
    /// Proven sandbox identity and lifetime once recovered or accepted.
    pub sandbox: Option<KnownSandbox>,
    /// Demand admission time while unsent, with no provider cost or lifetime claim.
    /// Dispatch replaces it with the actual dispatch time.
    /// Trusted provider creation time replaces it when measured.
    pub starts_at: DateTime<Utc>,
    /// Confirmed lifecycle end, independent of measured billing time.
    pub ends_at: Option<DateTime<Utc>>,
    /// Finite measured billing prefix, independent of lifecycle reconciliation.
    pub billing: Option<MeasuredBillingInterval>,
    /// Admission fee bound retained for its UTC start month without provider confirmation.
    pub start_fee_bound_usd: Option<Decimal>,
    /// Provider-confirmed actual start charge, never inferred from the saved bound.
    pub start_usd: Option<Decimal>,
    /// First accepted guest end report, absent before token consumption.
    pub reported_at: Option<DateTime<Utc>>,
    /// Exact per-calendar-month runtime observations from the provider.
    pub runtime: Vec<RuntimeCharge>,
    /// Trusted total runtime evidence, retained even when monthly allocation fails.
    pub runtime_total: Option<RuntimeTotal>,
    /// Latest trusted evidence that runtime was still running, including partial reads.
    pub running_observed_at: Option<DateTime<Utc>>,
    /// A confirmed delete can leave an unmeasured interval after its last read.
    pub runtime_complete: bool,
}

impl StartRecord {
    /// Select the start fee counted by spend and savings without confirming a charge.
    pub fn counted_start_fee_usd(&self) -> Option<Decimal> {
        self.start_usd.or(self.start_fee_bound_usd)
    }
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

/// Runtime dollars over the sandbox's complete observed lifetime.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RuntimeTotal {
    /// Trusted provider creation time.
    pub since: DateTime<Utc>,
    /// Last instant measured by the provider.
    pub until: DateTime<Utc>,
    /// Measured runtime dollars, excluding the start fee.
    pub usd: Decimal,
    /// Whether runtime still accrued at this observation.
    pub running: bool,
}
