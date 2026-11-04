//! Machine registration and last-report storage.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;

/// The most recent capacity report for one disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineDisk {
    /// The mount path reported by the agent.
    pub path: String,
    /// The total capacity in bytes.
    pub total_bytes: i64,
    /// The free capacity in bytes.
    pub free_bytes: i64,
}

/// A registered own machine and its latest report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Machine {
    /// The stable machine identifier.
    pub id: Uuid,
    /// The host name reported by the agent.
    pub hostname: String,
    /// The processor name reported by the agent.
    pub processor: String,
    /// The hardware thread count.
    pub hardware_threads: i32,
    /// The physical memory capacity in bytes.
    pub memory_bytes: i64,
    /// Whether KVM is available.
    pub kvm_available: bool,
    /// The slots reported by the agent.
    pub slots: i32,
    /// The desired count of warm runners.
    pub warm_runners: i32,
    /// Whether the machine should finish its jobs without taking new ones.
    pub draining: bool,
    /// The last contact, supplied by the gateway.
    pub last_contact_at: Option<DateTime<Utc>>,
    /// The last reported agent version.
    pub agent_version: String,
    /// The latest capacity of each disk.
    pub disks: Vec<MachineDisk>,
    /// When removal revoked this machine's credentials.
    pub removed_at: Option<DateTime<Utc>>,
}

/// Queries for machine registration, reports, drain, and removal.
pub trait MachineStore: Send + Sync {
    /// Insert a machine without a plaintext credential.
    fn create_machine(&self, workspace_id: Uuid, machine: &Machine) -> Result<()>;
    /// Read active registrations, including offline and draining machines.
    fn machines(&self, workspace_id: Uuid) -> Result<Vec<Machine>>;
    /// Replace a machine report and desired settings within its workspace.
    fn update_machine(&self, workspace_id: Uuid, machine: &Machine) -> Result<()>;
    /// Remove a machine and revoke every machine credential atomically.
    fn remove_machine(&self, workspace_id: Uuid, machine_id: Uuid, at: DateTime<Utc>)
    -> Result<()>;
}
