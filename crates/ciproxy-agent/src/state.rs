//! Root-only durable identity and VM ownership. Registrations never enter state.

use std::collections::{BTreeMap, BTreeSet};

use ciproxy_protocol::cache::DockerBudget;
use ciproxy_protocol::guest::JobPreparation;
use ciproxy_protocol::identifiers::{MachineId, WorkspaceId};
use ciproxy_protocol::runners::RunnerReport;
use ciproxy_protocol::secret::Secret;
use serde::{Deserialize, Serialize};

use crate::cache_authority::CacheAuthority;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// The workspace-bound machine credential saved on this machine.
pub struct Identity {
    pub(crate) workspace_id: WorkspaceId,
    pub(crate) machine_id: MachineId,
    pub(crate) credential: Secret,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
/// Durable admission state across service restarts.
pub enum Mode {
    /// The machine accepts new runners.
    #[default]
    Active,
    /// Jobs finish while idle runners stop.
    Draining,
    /// Credentials are revoked and the process exits after its jobs.
    Removed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Durable pre-action setup state. Started setup never runs again after recovery.
pub(crate) enum PreparationState {
    Waiting,
    Started,
    Completed(JobPreparation),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// A locally generated jail identity and its protocol report.
pub struct VmRecord {
    pub(crate) preparation: PreparationState,
    /// Required original non-secret destination and permission authority.
    pub(crate) cache_authority: CacheAuthority,
    /// CP-confirmed assignment metadata, without runtime credentials.
    pub(crate) job_facts: Option<ciproxy_protocol::jobs::JobFacts>,
    pub(crate) local_id: String,
    pub(crate) network_index: u16,
    pub(crate) report: RunnerReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegistrationClaim {
    Delivery,
    Retirement,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
/// The root-only state committed before publishing lifecycle changes.
pub struct MachineState {
    /// In-flight owners share the state lock but never survive a process restart.
    #[serde(skip)]
    pub(crate) registration_claims: BTreeMap<String, RegistrationClaim>,
    /// Failed starts retry VM and disk cleanup before releasing their record.
    #[serde(default)]
    pub(crate) failed_starts: BTreeSet<String>,
    /// Completed VMs release slots while their network indices remain owned.
    #[serde(default)]
    pub(crate) pending_networks: BTreeSet<u16>,
    /// Latest hosted spare-disk policy, persisted before any Docker attachment.
    pub(crate) docker_budget: Option<DockerBudget>,
    pub(crate) identity: Option<Identity>,
    pub(crate) mode: Mode,
    pub(crate) runners: Vec<VmRecord>,
}
