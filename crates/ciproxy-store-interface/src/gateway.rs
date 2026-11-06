//! Atomic machine gateway queries and full latest reports.

use chrono::{DateTime, Utc};
use ciproxy_protocol::reports::MachineReport;
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::machines::Machine;
use crate::secrets::TokenHash;
use crate::tokens::TokenStore;

/// The full last report and the machine's desired settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineSnapshot {
    /// Identity, desired settings, and last contact time.
    pub machine: Machine,
    /// Latest full report, absent for registrations made before the gateway.
    pub report: Option<ReceivedReport>,
}

/// One full inventory report bound to its trusted gateway receipt time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceivedReport {
    /// Complete inventory from one authenticated message.
    pub report: MachineReport,
    /// Receipt time of this inventory, independent of later lifecycle contact.
    pub received_at: DateTime<Utc>,
}

/// A fresh machine and its first authenticated connection.
#[derive(Clone, Debug)]
pub struct Registration {
    /// Workspace authorized by the install token.
    pub workspace_id: Uuid,
    /// Hash of the reusable install token.
    pub install_hash: TokenHash,
    /// Identity allocated for the new machine.
    pub machine_id: Uuid,
    /// Record identifier of the new machine credential.
    pub credential_id: Uuid,
    /// Hash of the machine credential returned once to the agent.
    pub credential_hash: TokenHash,
    /// First connection and report.
    pub contact: Contact,
}

/// A server-timed report fenced by the connection identifier.
#[derive(Clone, Debug)]
pub struct Contact {
    /// Identifier replaced on each authenticated connection.
    pub session_id: Uuid,
    /// Current machine state without credentials.
    pub report: MachineReport,
    /// Server receipt time, never an agent-supplied timestamp.
    pub at: DateTime<Utc>,
}

/// Synchronous queries used by the gateway from blocking workers.
#[unimock(api = GatewayStoreMock)]
pub trait GatewayStore: TokenStore + Send + Sync {
    /// Store an install hash with an exclusive expiry one hour after issuance.
    fn issue_install_token(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        hash: &TokenHash,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Revoke an install token in its workspace without changing machines.
    fn revoke_install_token(&self, workspace_id: Uuid, id: Uuid, at: DateTime<Utc>) -> Result<()>;
    /// Authenticate the install hash and insert the machine, report, and credential atomically.
    fn register_machine(&self, registration: &Registration) -> Result<bool>;
    /// Authenticate and replace the current session without changing desired settings.
    fn connect_machine(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        hash: &TokenHash,
        contact: &Contact,
    ) -> Result<Option<Machine>>;
    /// Replace the latest report only for the active, unremoved connection.
    fn record_contact(&self, workspace_id: Uuid, machine_id: Uuid, contact: &Contact)
    -> Result<()>;
    /// Record contact from a lifecycle message without replacing the last full report.
    fn record_activity(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        session_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Read the current connection of an unremoved machine.
    fn machine_session(&self, workspace_id: Uuid, machine_id: Uuid) -> Result<Option<Uuid>>;
    /// Read active machines, disks, and last full reports from one committed state.
    fn machine_snapshots(&self, workspace_id: Uuid) -> Result<Vec<MachineSnapshot>>;
    /// Change desired drain state without changing observed reports.
    fn set_machine_draining(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        draining: bool,
    ) -> Result<()>;
    /// Remove the machine and revoke its credentials atomically, including while offline.
    fn revoke_machine(&self, workspace_id: Uuid, machine_id: Uuid, at: DateTime<Utc>)
    -> Result<()>;
}
