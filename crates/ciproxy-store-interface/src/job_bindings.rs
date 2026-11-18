//! Exact Worker job identities bound to authenticated own-machine runners.

use ciproxy_protocol::identifiers::JobId;
use uuid::Uuid;

/// Worker job identity, separate from scale-set UUIDs and numeric Actions IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerJobId(#[doc = "The exact identity carried by the runner's job report."] pub JobId);

/// Complete authenticated scope for one immutable own-machine job binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnJobBinding {
    /// Exact internal runner identity that owns the job.
    pub runner_id: Uuid,
    /// Organization that owns the runner registration.
    pub organization_id: Uuid,
    /// Authenticated machine that holds the runner placement.
    pub machine_id: Uuid,
    /// GitHub runner identity already saved for this registration.
    pub github_runner_id: i64,
    /// Exact Worker job identity reported by this runner.
    pub worker_job_id: WorkerJobId,
}
