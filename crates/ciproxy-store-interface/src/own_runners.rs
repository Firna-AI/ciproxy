//! Durable own-machine launch and retirement delivery records.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;
use crate::gateway::MachineSnapshot;
use crate::github::GithubJob;
use crate::labels::Label;
use crate::organizations::Organization;
use crate::runners::Runner;
use crate::settings::Settings;

/// One credential-free command paired with its immutable runner placement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnRunner {
    /// Original label request in byte units, retained across settings changes.
    pub requested_size: crate::vm_size::VmSize,
    /// Exact logical slots held by this VM, including non-contiguous sets.
    pub slot_indices: Vec<u64>,
    /// Immutable placement and current hosted lifecycle.
    pub runner: Runner,
    /// Label name at launch, including after the label is removed.
    pub label_name: String,
    /// Current start or stop command identity.
    pub command_id: Uuid,
    /// Trusted issue time used to fence older full machine reports.
    pub issued_at: DateTime<Utc>,
    /// Whether the machine accepted this command.
    pub acknowledged: bool,
}

/// Queries used by the own-machine runtime, without storing JIT secrets.
pub trait OwnRunnerStore: Send + Sync {
    /// Reconcile exact original and actual dimensions from the authenticated VM.
    fn save_own_size(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        requested_size: crate::vm_size::VmSize,
        allocation: ciproxy_protocol::resources::VmSize,
    ) -> Result<()>;
    /// Reconcile the exact durable slot set reported by the owning agent.
    fn save_own_slots(&self, workspace_id: Uuid, runner_id: Uuid, slots: &[u64]) -> Result<()>;
    /// Read routing inputs from one committed repeatable-read database snapshot.
    fn own_routing_snapshot(&self, workspace_id: Uuid) -> Result<RoutingSnapshot>;
    /// Claim this workspace's execution decisions on a dedicated database session.
    fn try_lock_own_runners(&self, workspace_id: Uuid) -> Result<Option<Box<dyn OwnRunnerLease>>>;
    /// Discover workspaces with retained execution resources after restart.
    fn routing_workspaces(&self) -> Result<Vec<Uuid>>;
    /// Read every own-machine intent in its workspace.
    fn own_runners(&self, workspace_id: Uuid) -> Result<Vec<OwnRunner>>;
    /// Atomically insert a reserved runner and its first command.
    fn create_own_runner(&self, workspace_id: Uuid, runner: &OwnRunner) -> Result<()>;
    /// Persist a stop intent after GitHub confirms registration removal.
    fn retire_own_runner(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        command_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Confirm one current command without accepting a superseded result.
    fn acknowledge_own_runner(&self, workspace_id: Uuid, command_id: Uuid) -> Result<()>;
}

/// Coherent persisted input to one workspace's pure routing calculation.
pub struct RoutingSnapshot {
    /// Workspace that owns every resource below.
    pub workspace_id: Uuid,
    /// Current and retained organization registrations.
    pub organizations: Vec<Organization>,
    /// Current label settings.
    pub labels: Vec<Label>,
    /// Full reports and separately stored desired machine settings.
    pub machines: Vec<MachineSnapshot>,
    /// Durable launch and retirement intents.
    pub runners: Vec<OwnRunner>,
    /// Trusted scale-set demand and recent completed history.
    pub jobs: Vec<GithubJob>,
    /// Shared routing settings.
    pub settings: Settings,
}

/// Exclusive workspace execution ownership retained across remote operations.
pub trait OwnRunnerLease: Send {
    /// Check that the database session still owns this workspace.
    fn check_connection(&mut self) -> Result<()>;
}
