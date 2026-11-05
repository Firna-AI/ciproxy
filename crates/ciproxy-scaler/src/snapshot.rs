//! Complete immutable facts for one workspace at one caller-supplied time.

use std::time::Duration;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::{
    JobId, LabelName, MachineId, OrganizationId, RunnerId, WorkspaceId,
};
use ciproxy_protocol::resources::{Slots, VmSize};

use crate::boat::{BoatAccount, Sandbox};

/// One workspace's routing inputs, including all starts still in flight.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    /// Workspace that owns every supplied machine and marked sandbox.
    pub workspace_id: WorkspaceId,
    /// Logical image length for the release used by every new runner.
    pub root_sizing: RootSizing,
    /// Current time on the same clock as queue and retry timestamps.
    pub now: Duration,
    /// Whether the control plane can currently apply routing actions.
    pub control_plane_available: bool,
    /// Organizations that may receive new runners in this workspace.
    pub organizations: Vec<OrganizationId>,
    /// Current workspace labels.
    pub labels: Vec<Label>,
    /// Waiting jobs in caller order, excluding jobs already running.
    pub queue: Vec<WaitingJob>,
    /// Complete own-machine state, including occupied and retiring VMs.
    pub machines: Vec<Machine>,
    /// Boat settings and the current account catalog.
    pub boat: BoatAccount,
    /// Sandboxes observed by the backend, including marked orphans.
    pub sandboxes: Vec<Sandbox>,
    /// Time that eligible own-machine jobs wait before an overflow start.
    pub overflow_wait: Duration,
}

/// Routing settings shared by all organizations of one workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    /// GitHub scale set label.
    pub name: LabelName,
    /// Current label dimensions, with disk measured as free job bytes.
    pub size: VmSize,
    /// Destinations enabled by the customer.
    pub destinations: Destinations,
    /// Behavior when neither own machines nor the Boat catalog fits.
    pub nothing_fits: NothingFits,
}

/// The two independent label destination settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Destinations {
    /// Whether the label allows own machines.
    pub own_machines: bool,
    /// Whether the label allows Boat.
    pub boat: bool,
}

/// The customer's choice for a size unsupported by both destinations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NothingFits {
    /// Leave demand queued until a destination of the requested size exists.
    Wait,
    /// Permit the largest Boat size, even when it is smaller than requested.
    LargestBoatSize,
}

/// Unassigned demand, which the scaler never pins to a particular runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitingJob {
    /// Organization whose label has waiting demand.
    pub organization_id: OrganizationId,
    /// Label whose current dimensions are requested.
    pub label: LabelName,
    /// Time when GitHub assigned the waiting job to the scale set.
    pub queued_at: Duration,
}

/// Capacity available to all organizations of the workspace on one machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Machine {
    /// Own-machine identity.
    pub id: MachineId,
    /// Whether the machine is connected to the control plane.
    pub connected: bool,
    /// Whether the machine must accept no new runner.
    pub draining: bool,
    /// Total slots after the protocol's CPU and memory host reserve.
    pub capacity: Slots,
    /// Total job disk after the host reserve, including cache space.
    ///
    /// This includes disk held by every runner below. It is not free job disk.
    /// The machine cache and unused Docker disks free space for new job VMs.
    pub disk_budget_bytes: u64,
    /// One shared warm count for this machine, across all organizations.
    pub warm_count: usize,
    /// Per-machine targets from the documented warm-runner allocation policy.
    ///
    /// The snapshot builder applies "Warm runners" in job-routing.md.
    /// One entry means one desired warm runner. The scaler caps the list at
    /// `warm_count`.
    pub warm_allocation: Vec<OrganizationId>,
    /// All VMs whose slots and disk have not yet been released.
    pub runners: Vec<Runner>,
}

/// A live or retiring runner whose original size stays fixed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Runner {
    /// Identity allocated when the caller applied its start intent.
    pub id: RunnerId,
    /// Organization whose registration owns this runner.
    pub organization_id: OrganizationId,
    /// Label retained from registration.
    pub label: LabelName,
    /// Label dimensions at registration, used to detect later size edits.
    pub requested_size: VmSize,
    /// Actual VM size retained for its lifetime, including the full root disk.
    pub size: VmSize,
    /// Last confirmed lifecycle state.
    pub state: RunnerState,
}

/// Lifecycle facts that keep starting capacity and running jobs safe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunnerState {
    /// A start is persisted but the runner is not ready yet.
    Starting,
    /// GitHub can assign a job to this idle runner.
    Ready,
    /// The runner owns a job and must never be retired by the scaler.
    Running(JobId),
    /// Retirement is pending and the VM still holds capacity.
    Retiring,
    /// The process exited and no job is still running.
    Exited,
}
