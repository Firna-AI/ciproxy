//! Durable own-machine launch and retirement delivery records.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::boat_runtime::BoatRunnerIntent;
use crate::error::Result;
use crate::gateway::MachineSnapshot;
use crate::github::GithubJob;
use crate::labels::Label;
use crate::organizations::Organization;
use crate::overflow::store::OverflowSnapshot;
use crate::runners::{Runner, RunnerState};
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
    /// Trusted issue time of the current command.
    pub issued_at: DateTime<Utc>,
    /// Whether the machine accepted this command.
    pub acknowledged: bool,
    /// First command result or trusted agent/GitHub observation of this runner.
    /// Retirement preserves this evidence for the VM's complete lifetime.
    pub confirmed_at: Option<DateTime<Utc>>,
}

/// One trusted execution observation, saved together with its first evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnRunnerObservation {
    /// Effective hosted state after the caller preserves job and stop protections.
    pub state: RunnerState,
    /// Saved identity or the trusted GitHub identity supplied by this observation.
    pub github_id: Option<i64>,
    /// Full-report receipt time or the hosted clock time of another observation.
    pub observed_at: DateTime<Utc>,
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
        github_id: Option<i64>,
    ) -> Result<()>;
    /// Atomically save state, identity, and first evidence within the runner scope.
    fn observe_own_runner(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        runner_id: Uuid,
        observation: OwnRunnerObservation,
    ) -> Result<()>;
    /// Record current-command evidence and close delivery only for an applied result.
    fn record_own_command_result(
        &self,
        workspace_id: Uuid,
        command_id: Uuid,
        applied: bool,
        at: DateTime<Utc>,
    ) -> Result<()>;
}

/// Coherent persisted input to one workspace's pure routing calculation.
pub struct RoutingSnapshot {
    /// Boat account and delivery facts from this same committed snapshot.
    pub overflow: OverflowSnapshot,
    /// Registration identities saved before GitHub JIT requests.
    pub boat_intents: Vec<BoatRunnerIntent>,
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
