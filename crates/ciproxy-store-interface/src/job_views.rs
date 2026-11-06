//! Credential-free job projections across Actions and scale-set attempts.

use chrono::{DateTime, Utc};
use ciproxy_protocol::resources::VmSize;
use serde::Deserialize;
use uuid::Uuid;

use crate::jobs::{CacheSource, JobState, MachineHandledCacheCounts};

/// One retained job with trusted placement and its organization identity.
#[derive(Clone, Debug, Deserialize)]
pub struct DashboardJob {
    /// Stable Actions or exact scale-set attempt identity.
    pub id: Uuid,
    /// Organization whose repository owns the job.
    pub organization_id: Uuid,
    /// Current organization login.
    pub organization: String,
    /// Exact retained label identity, independent of a later same-name label.
    pub label_id: Uuid,
    /// Numeric Actions identity when GitHub supplied it.
    pub actions_job_id: Option<i64>,
    /// Workflow-run identity when GitHub supplied it.
    pub run_id: Option<i64>,
    /// Job name from GitHub.
    pub name: String,
    /// Workflow name or file identity from GitHub.
    pub workflow: String,
    /// Owner and repository from GitHub.
    pub repository: String,
    /// Full ref only when the trusted source supplied one.
    pub git_ref: Option<String>,
    /// GitHub's branch or tag name without an invented ref namespace.
    pub head_branch: Option<String>,
    /// Latest job state.
    pub state: JobState,
    /// Trusted lifecycle and runner identity prove a start, independent of time.
    pub runner_taken: bool,
    /// Verbatim source result. REST and scale-set vocabularies stay unchanged.
    pub result: Option<String>,
    /// Queue assignment time, absent when GitHub supplied none.
    pub queued_at: Option<DateTime<Utc>>,
    /// Real runner-taken time, absent before a start or until timing recovery.
    pub started_at: Option<DateTime<Utc>>,
    /// Real GitHub finish, absent for unknown terminal timing.
    pub completed_at: Option<DateTime<Utc>>,
    /// Requested label name.
    pub label: String,
    /// Requested label CPU count.
    pub requested_vcpus: i32,
    /// Requested label memory in decimal GB.
    pub requested_memory_gb: i32,
    /// Requested free job disk in decimal GB.
    pub requested_disk_gb: i32,
    /// Runner's immutable resources, absent until placement is known.
    pub actual_size: Option<VmSize>,
    /// Own-machine identity, absent for Boat or unknown placement.
    pub machine_id: Option<Uuid>,
    /// Own-machine hostname, including removed machines.
    pub machine_name: Option<String>,
    /// First assigned own-machine slot.
    pub first_slot: Option<i32>,
    /// Actual Boat size name, absent for an own machine.
    pub boat_size: Option<String>,
    /// Saved reason when the trusted routing service supplied one.
    pub routing_reason: Option<String>,
    /// Restored content origins, absent for zero or unknown restores.
    pub cache_source: Option<CacheSource>,
    /// Counts only when the machine cache handled the restore.
    pub cache_counts: Option<MachineHandledCacheCounts>,
}
