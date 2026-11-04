//! Root-only durable identity and VM ownership. Registrations never enter state.

use ciproxy_protocol::identifiers::{MachineId, WorkspaceId};
use ciproxy_protocol::runners::RunnerReport;
use ciproxy_protocol::secret::Secret;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// A locally generated jail identity and its protocol report.
pub struct VmRecord {
    pub(crate) local_id: String,
    pub(crate) network_index: u16,
    pub(crate) report: RunnerReport,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
/// The root-only state committed before publishing lifecycle changes.
pub struct MachineState {
    pub(crate) identity: Option<Identity>,
    pub(crate) mode: Mode,
    pub(crate) runners: Vec<VmRecord>,
}
