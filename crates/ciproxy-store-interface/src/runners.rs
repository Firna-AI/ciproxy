//! Runner placement, resource snapshots, and lifecycle records.

use uuid::Uuid;

use crate::error::Result;
use ciproxy_protocol::resources::VmSize;

/// The stored lifecycle state of a runner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum RunnerState {
    /// The VM and GitHub registration are starting.
    Starting,
    /// The runner can take one job.
    Ready,
    /// The runner is executing its job.
    Busy,
    /// The runner is shutting down.
    Stopping,
    /// The VM or sandbox has ended.
    Stopped,
}

/// The resource that owns one runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunnerPlace {
    /// A job VM on one own machine.
    Machine {
        /// The workspace's machine identifier.
        machine_id: Uuid,
        /// The first slot assigned to the VM.
        first_slot: i32,
    },
    /// A sandbox in the workspace's Boat account.
    Boat {
        /// The retained key version, absent after the key is erased.
        key_id: Option<Uuid>,
        /// The Boat sandbox identifier.
        sandbox_id: String,
        /// The actual Boat VM size name.
        size_name: String,
    },
}

/// A runner with its immutable placement and actual VM resources.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Runner {
    /// The internal runner identifier.
    pub id: Uuid,
    /// The GitHub organization that owns this runner registration.
    pub organization_id: Uuid,
    /// The label that the runner serves.
    pub label_id: Uuid,
    /// The scale set that owns the GitHub registration.
    pub scale_set_id: Uuid,
    /// The GitHub runner identifier after registration.
    pub github_id: Option<i64>,
    /// The current lifecycle state.
    pub state: RunnerState,
    /// The actual resources of its VM.
    pub size: VmSize,
    /// Where the runner lives.
    pub place: RunnerPlace,
}

/// Tenant-scoped runner records used by later lifecycle services.
pub trait RunnerStore: Send + Sync {
    /// Reconcile byte-precise actual resources without changing original placement.
    fn update_runner_size(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        id: Uuid,
        size: VmSize,
    ) -> Result<()>;
    /// Insert one runner without its GitHub registration secret.
    fn create_runner(&self, workspace_id: Uuid, runner: &Runner) -> Result<()>;
    /// Read all retained runners of one organization.
    fn runners(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<Runner>>;
    /// Record a lifecycle state and the optional GitHub runner identifier.
    fn update_runner(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        id: Uuid,
        state: RunnerState,
        github_id: Option<i64>,
    ) -> Result<()>;
}
