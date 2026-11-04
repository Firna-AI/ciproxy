//! Job and registration facts read across every organization of a workspace.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::metering::totals::{AccountedJob, StartTotal};
use crate::vm_size::VmSize;

/// The execution location kept with a metered job.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobPlace {
    /// A VM on a customer's own machine.
    Machine,
    /// A sandbox that ciproxy created in Boat.
    Boat,
}

/// One runner-taken job, including running and failed jobs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageJob {
    /// The stable internal identifier, distinct for each re-run.
    pub id: Uuid,
    /// The organization whose repository owns this job.
    pub organization_id: Uuid,
    /// When the runner took this job.
    pub started_at: DateTime<Utc>,
    /// When GitHub completed it, absent while it runs.
    pub completed_at: Option<DateTime<Utc>>,
    /// The immutable actual resources of the runner VM.
    pub size: VmSize,
    /// The immutable execution location.
    pub place: JobPlace,
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
