//! Own-machine views, health, cache facts, and Admin controls.

use super::money::Cents;
use crate::access::workspace::WorkspaceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Hardware and state for one own machine. Byte counts use decimal strings.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct MachineView {
    /// Stable registered-machine identity.
    pub id: String,
    /// Host name reported by the agent.
    pub hostname: String,
    /// Processor model.
    pub processor: String,
    /// Reported hardware threads.
    pub threads: i32,
    /// Installed memory in bytes.
    pub memory_bytes: String,
    /// Whether KVM can start job VMs.
    pub kvm: bool,
    /// Reported CPU and memory slot capacity.
    pub slots: i32,
    /// Full root disk of this machine's selected image, absent when unknown.
    pub selected_image_disk_bytes: Option<String>,
    /// Whole default-label jobs that fit concurrently, including disk.
    pub concurrent_default_jobs: Option<String>,
    /// Whether prepared capacity and the required root allocation are available.
    pub capacity_known: bool,
    /// Whether no contact arrived during the last sixty seconds.
    pub offline: bool,
    /// Whether health values describe old contact data.
    pub health_old: bool,
    /// Desired drain state.
    pub draining: bool,
    /// Desired count of warm runners.
    pub warm_runners: i32,
    /// Last server-observed contact time.
    pub last_contact_at: Option<DateTime<Utc>>,
    /// Last reported agent release.
    pub agent_version: String,
    /// Last reported disks.
    pub disks: Vec<DiskView>,
    /// Last reported individual slot occupants.
    pub occupancy: Vec<SlotView>,
    /// Last reported machine-cache facts.
    pub cache: Option<CacheView>,
    /// Current monthly machine cost in integer US cents.
    pub monthly_cost_cents: Option<Cents>,
}

/// Last reported disk capacity and free space.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct DiskView {
    /// Reported mount path or name.
    pub path: String,
    /// Total bytes as a decimal string.
    pub capacity_bytes: String,
    /// Free bytes as a decimal string.
    pub free_bytes: String,
}

/// Last reported occupancy of one slot.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct SlotView {
    /// Zero-based slot index as a decimal string.
    pub index: String,
    /// Free, warm, or job occupancy.
    pub state: SlotState,
    /// Registered runner identity, absent for a free slot.
    pub runner_id: Option<String>,
    /// Job identity, present only for a running job.
    pub job_id: Option<String>,
}

/// The closed set of slot occupancy states.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SlotState {
    /// No runner occupies this slot.
    Free,
    /// A starting or warm runner occupies this slot.
    Warm,
    /// A job occupies this slot.
    Job,
}

/// Machine-cache metadata. Clear cache acts only on this machine's entries.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CacheView {
    /// Machine-cache bytes used, as a decimal string.
    pub used_bytes: String,
    /// Machine-cache capacity bytes, as a decimal string.
    pub capacity_bytes: String,
    /// Handled restores during the last seven days.
    pub restores_last_7_days: String,
    /// Local hits during the last seven days.
    pub hits_last_7_days: String,
    /// Persistent Docker disk bytes, which Clear cache preserves.
    pub docker_disk_bytes: String,
}

/// Independent desired warm-runner target.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct WarmRunnerTarget {
    /// Desired warm runner count. It does not change monthly cost.
    pub warm_runners: i32,
}

/// Current monthly cost entered independently from the warm-runner target.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct MachineCostInput {
    /// Integer US cents. Known zero remains distinct from an unknown cost.
    /// The interval starts at entry time and supplies no earlier historical price.
    pub monthly_cost_cents: Cents,
}

/// One workspace-owned machine.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct MachinePath {
    /// Stable workspace identity.
    #[param(value_type=Uuid)]
    pub workspace_id: WorkspaceId,
    /// Stable machine identity.
    pub machine_id: Uuid,
}

/// One install-token revocation target.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct TokenPath {
    /// Stable workspace identity.
    #[param(value_type=Uuid)]
    pub workspace_id: WorkspaceId,
    /// Opaque token record identity.
    pub token_id: Uuid,
}

/// Install command returned to an Admin only.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct InstallCommand {
    /// Record identity used for early revocation.
    pub id: String,
    /// Single shell command containing the one-hour install secret.
    pub command: String,
    /// Exclusive token expiry in UTC.
    pub expires_at: DateTime<Utc>,
    /// Machine requirements displayed next to the command.
    pub requirements: String,
}

/// Result of immediate gateway command delivery.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CommandResult {
    /// Whether the active local agent connection accepted the command.
    pub connected: bool,
}

/// Admin command carried by the machine control route.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MachineCommand {
    /// Finish jobs and stop accepting new jobs.
    Drain,
    /// Resume taking new jobs and keeping warm runners.
    Resume,
    /// Revoke the machine immediately, also when offline.
    Remove,
    /// Clear only this machine's cache entries.
    ClearCache,
}

/// A machine and one closed-set Admin action.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct MachineCommandPath {
    /// Stable workspace identity.
    #[param(value_type=Uuid)]
    pub workspace_id: WorkspaceId,
    /// Stable machine identity.
    pub machine_id: Uuid,
    /// Documented gateway action.
    #[param(inline)]
    pub action: MachineCommand,
}
