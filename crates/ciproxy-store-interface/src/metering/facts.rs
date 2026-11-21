//! Job and registration facts read across every organization of a workspace.

use chrono::{DateTime, Utc};
use ciproxy_protocol::resources::VmSize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::metering::totals::{AccountedJob, StartTotal};

/// Explicit metadata source; a UUID alone never aliases two source namespaces.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum JobSource {
    /// Existing Actions-history job record.
    Actions,
    /// Durable scale-set UUID demand with trusted runner mapping.
    ScaleSet,
}

/// Trusted terminal timing; unavailable finish facts never mean still running.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(tag = "state", content = "at", rename_all = "snake_case")]
pub enum Completion {
    /// GitHub still reports this attempt in progress.
    Running,
    /// GitHub supplied the actual completion time.
    Finished(DateTime<Utc>),
    /// GitHub ended the attempt without supplying a completion time.
    TerminalUnknown,
}
/// The execution location kept with a metered job.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobPlace {
    /// A VM on a customer's own machine.
    Machine,
    /// A sandbox that ciproxy created in Boat.
    Boat,
}

/// One runner-taken job, including running and failed jobs.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct UsageJob {
    /// Namespace that owns this metadata identity.
    pub source: JobSource,
    /// The stable internal identifier, distinct for each re-run.
    pub id: Uuid,
    /// The organization whose repository owns this job.
    pub organization_id: Uuid,
    /// When the runner took this job.
    pub started_at: DateTime<Utc>,
    /// Trusted completion state and real finish time, when available.
    pub completion: Completion,
    /// The immutable actual resources of the runner VM.
    pub size: VmSize,
    /// The immutable execution location.
    pub place: JobPlace,
}

impl UsageJob {
    /// Preserve source and organization when correlating accounting cursors.
    pub fn key(&self) -> (JobSource, Uuid, Uuid) {
        (self.source, self.organization_id, self.id)
    }
}

/// A current or removed registration, independent of machine health.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineRegistration {
    /// The stable own-machine identifier.
    pub machine_id: Uuid,
    /// The trusted registration time, absent for legacy records.
    pub registered_at: Option<DateTime<Utc>>,
    /// The removal time, absent while registered.
    pub removed_at: Option<DateTime<Utc>>,
}

/// Workspace facts required for counts and runtime calculations.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceUsage {
    /// Scale attempts explicitly mapped to an existing numeric job and same runner.
    pub aliases: Vec<AliasedJob>,
    /// Jobs with runner assignments and a start time.
    pub jobs: Vec<UsageJob>,
    /// Anonymous taken-at counts, including deleted organizations' starts.
    pub starts: Vec<StartTotal>,
    /// Previous facts for live jobs already included in anonymous counts.
    pub accounted_jobs: Vec<AccountedJob>,
    /// Current and removed own-machine registrations.
    pub machines: Vec<MachineRegistration>,
    /// Organizations whose App remains active or suspended.
    pub organizations: u64,
}

/// One duplicate projection whose prior accounting must be retracted exactly once.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AliasedJob {
    /// The scale-attempt accounting identity.
    pub id: Uuid,
    /// Its retained owning organization.
    pub organization_id: Uuid,
}
