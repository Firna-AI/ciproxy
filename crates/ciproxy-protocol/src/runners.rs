//! Runner launch values and reconnect state for fresh job VMs.

use serde::{Deserialize, Serialize};

use crate::identifiers::{ImageId, JobId, LabelName, OrganizationId, RunnerId};
use crate::resources::VmSize;
use crate::secret::Secret;

/// Everything the agent needs to start one ephemeral runner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunnerSpec {
    /// The runner and VM identity assigned by the control plane.
    pub runner_id: RunnerId,
    /// GitHub organization whose scale set owns this runner registration.
    pub organization_id: OrganizationId,
    /// Label whose jobs this runner can take.
    pub label: LabelName,
    /// Original label resources, with disk measured as free job bytes.
    pub size: VmSize,
    /// Just-in-time registration valid for one runner only.
    pub registration: Secret,
}

/// A runner that the agent still owns, also after a reconnect.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunnerReport {
    /// The runner and VM identity.
    pub runner_id: RunnerId,
    /// Organization scope retained by this runner across agent reconnects.
    pub organization_id: OrganizationId,
    /// Label used when the runner started.
    pub label: LabelName,
    /// Original label resources, retained independently of actual allocation.
    pub requested_size: VmSize,
    /// Actual VM resources retained until removal, even after a label edit.
    pub size: VmSize,
    /// Image retained by this VM for its whole lifetime.
    pub image_id: ImageId,
    /// The last state observed by the agent.
    pub state: RunnerState,
}

/// A runner's lifecycle without job logs or secrets.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RunnerState {
    /// The VM or its runner is still starting.
    Starting,
    /// The runner is ready and has not taken a job.
    Warm,
    /// The runner has taken its one job.
    Running {
        /// GitHub identity of that job.
        job_id: JobId,
    },
    /// The idle runner or finished VM is stopping.
    Stopping,
}

/// A runner process exit, independent of the job result reported by GitHub.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunnerExit {
    /// Process exit code, absent when the process ended without one.
    pub code: Option<i32>,
}
