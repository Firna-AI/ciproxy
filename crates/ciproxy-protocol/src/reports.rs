//! Machine hardware, slot occupancy, and state used by hello and reports.

use serde::{Deserialize, Serialize};

use crate::cache::{DockerDiskReport, MachineCacheReport};
use crate::identifiers::{JobId, RunnerId};
use crate::images::ImageReport;
use crate::resources::Slots;
use crate::runners::RunnerReport;

/// The documented report interval used by the agent, in seconds.
pub const REPORT_INTERVAL_SECONDS: u64 = 10;

/// The current machine state, also sent in full after a reconnect.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MachineReport {
    /// Host name shown on the dashboard.
    pub hostname: String,
    /// Installed agent release used for the version health check.
    pub agent_version: String,
    /// CPU, memory, KVM, and disk health observed on this machine.
    pub hardware: HardwareReport,
    /// Slot capacity calculated from the documented host reserve.
    pub capacity: Slots,
    /// Total job disk after this machine's persisted image budget and reserve.
    pub job_disk_bytes: u64,
    /// Current occupancy of the machine's individual slots.
    pub slots: Vec<SlotReport>,
    /// Every runner still owned by the agent.
    pub runners: Vec<RunnerReport>,
    /// Whether the machine accepts new runners.
    pub drain: DrainState,
    /// Machine cache space, seven-day hits, and per-job usage.
    pub cache: MachineCacheReport,
    /// Persistent repository Docker disk use.
    pub docker_disks: DockerDiskReport,
    /// Images available for new VMs.
    pub images: ImageReport,
}

/// Hardware and current health facts, without policy decisions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareReport {
    /// Processor model reported by the operating system.
    pub processor: String,
    /// Hardware threads, each equivalent to one vCPU.
    pub threads: u32,
    /// Installed memory, in bytes.
    pub memory_bytes: u64,
    /// Whether KVM is available to run job VMs.
    pub kvm: bool,
    /// Disk capacity and current free space.
    pub disks: Vec<DiskReport>,
}

/// Space and identity of a disk on an own machine.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiskReport {
    /// Disk name reported by the operating system.
    pub name: String,
    /// Total disk space, in bytes.
    pub capacity_bytes: u64,
    /// Current free disk space, in bytes.
    pub free_bytes: u64,
}

/// One slot's current occupant.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SlotReport {
    /// Zero-based index within this machine.
    pub index: u64,
    /// A free slot, a warm runner, or a job.
    pub occupancy: SlotOccupancy,
}

/// The states shown for a slot on the machine page.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SlotOccupancy {
    /// No runner occupies the slot.
    Free,
    /// A runner with no job occupies the slot, including while it starts.
    Warm {
        /// The runner whose VM occupies this slot.
        runner_id: RunnerId,
    },
    /// A running job occupies the slot.
    Job {
        /// The runner whose VM occupies this slot.
        runner_id: RunnerId,
        /// The job that this runner took.
        job_id: JobId,
    },
}

/// The machine's reversible drain state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DrainState {
    /// The machine may take new jobs and keep warm runners.
    Active,
    /// Running jobs finish, and the machine keeps no warm runners.
    Draining,
}
