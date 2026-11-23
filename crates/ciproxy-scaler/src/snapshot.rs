//! Complete immutable facts for one workspace at one caller-supplied time.

use std::time::Duration;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::{
    JobId, LabelName, MachineId, OrganizationId, RunnerId, WorkspaceId,
};
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::{Slots, VmSize};

use crate::provider::{ProviderAccount, ProviderId, Sandbox};
use crate::registration::RegistrationTarget;

/// One workspace's routing inputs, including all starts still in flight.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    /// Workspace that owns every supplied machine and marked sandbox.
    pub workspace_id: WorkspaceId,
    /// Fallback root sizing until an own machine reports its image.
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
    /// Workspace switch that allows new starts on overflow providers.
    pub overflow_enabled: bool,
    /// Current provider accounts with their catalogs and start requirements.
    pub providers: Vec<ProviderAccount>,
    /// Sandboxes observed by providers, including marked orphans.
    pub sandboxes: Vec<Sandbox>,
    /// Time a job waits for a fitting own machine before trying a later place.
    pub overflow_wait: Duration,
}

impl Snapshot {
    pub(crate) fn provider(&self, id: ProviderId) -> Option<&ProviderAccount> {
        self.providers.iter().find(|provider| provider.id == id)
    }
}

/// The one label that warm runners serve on a machine of this platform.
pub fn warm_label(platform: Platform) -> &'static str {
    match platform {
        Platform::LinuxX64 => "ciproxy",
        Platform::MacosArm64 => "ciproxy-macos",
    }
}

/// Routing settings shared by all organizations of one workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    /// Saved active bindings that can serve this label in each organization.
    pub targets: Vec<RegistrationTarget>,
    /// GitHub scale set label.
    pub name: LabelName,
    /// Platform of the label's job VMs, fixed when the label is created.
    pub platform: Platform,
    /// Current label dimensions, with disk measured as free job bytes.
    pub size: VmSize,
    /// Places that may serve this label, in the customer's order.
    pub order: Vec<Place>,
    /// What to do when no place of the order fits the size.
    pub nothing_fits: NothingFits,
}

impl Label {
    pub(crate) fn places(&self) -> impl Iterator<Item = Place> + '_ {
        self.order
            .iter()
            .copied()
            .filter(|place| self.platform == Platform::LinuxX64 || *place == Place::OwnMachines)
    }

    pub(crate) fn allows(&self, place: Place) -> bool {
        self.places().any(|candidate| candidate == place)
    }

    pub(crate) fn own_machines_first(&self) -> bool {
        self.places().next() == Some(Place::OwnMachines)
    }
}

/// One place in a label's order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Place {
    /// The workspace's own machines.
    OwnMachines,
    /// One provider that can start a sandbox for this label.
    Provider(ProviderId),
}

/// The customer's choice for a size unsupported by the places in the order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NothingFits {
    /// Leave demand queued until a destination of the requested size exists.
    Wait,
    /// Permit a provider's largest size, even when it is smaller than requested.
    LargestProviderSize,
}

/// Unassigned demand, which the scaler never pins to a particular runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitingJob {
    /// Saved binding that owns this waiting demand.
    pub target: RegistrationTarget,
    /// Time when GitHub assigned the waiting job to the scale set.
    pub queued_at: Duration,
}

/// Capacity available to all organizations of the workspace on one machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Machine {
    /// Own-machine identity.
    pub id: MachineId,
    /// Platform reported at registration. Only labels of this platform fit.
    pub platform: Platform,
    /// Exact selected local release, absent until reported by the machine.
    pub selected_image: Option<ImageMetadata>,
    /// Whether the machine is connected to the control plane.
    pub connected: bool,
    /// Whether the machine must accept no new runner.
    pub draining: bool,
    /// Total slots after the protocol's CPU and memory host reserve.
    pub capacity: Slots,
    /// Gross job disk after the host reserve, before active Docker reservations.
    ///
    /// This includes disk held by every runner below. It is not free job disk.
    /// The machine cache and unused Docker disks free space for new job VMs.
    pub disk_budget_bytes: u64,
    /// Active Docker reservations, absent while their statistics are unavailable.
    pub docker_leased_bytes: Option<u64>,
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

impl Machine {
    /// Prefer selected local release facts to the deployment fallback.
    pub fn root_sizing(&self, fallback: RootSizing) -> RootSizing {
        self.selected_image
            .as_ref()
            .map_or(fallback, RootSizing::for_image)
    }
}

/// A live or retiring runner whose original size stays fixed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Runner {
    /// Captured registration identity. Older unrecorded targets stay unknown.
    pub target: Option<RegistrationTarget>,
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
