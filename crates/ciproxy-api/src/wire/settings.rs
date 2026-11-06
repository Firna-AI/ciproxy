//! Non-Boat setup, labels, and per-organization repository policy.

use crate::access::workspace::WorkspaceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Non-Boat setup progress for one workspace.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct SetupView {
    /// Current release image disk metadata, separate from machine-selected images.
    pub current_image_disk_bytes: String,
    /// Installed organizations and their current App states.
    pub organizations: Vec<OrganizationView>,
    /// Current registered machine count.
    pub machines: u32,
    /// Whether the page still waits for its first machine.
    pub waiting_for_machine: bool,
    /// The first observed job, absent while no job has entered this workspace.
    pub first_job: Option<SetupJob>,
    /// Workspace workflow labels.
    pub labels: Vec<LabelView>,
}

/// Admin-only non-Boat Settings data.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct SettingsView {
    /// Workspace name editable by an Admin.
    pub name: String,
    /// Current organizations, including owner-removed records during retention.
    pub organizations: Vec<OrganizationView>,
    /// Active labels and their computed machine fit.
    pub labels: Vec<LabelView>,
    /// Count displayed on the Members tab.
    pub open_requests: u32,
    /// Whether the current tier permits another organization.
    pub can_add_organization: bool,
}

/// A workspace rename request.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct WorkspaceName {
    /// Customer-visible workspace name.
    pub name: String,
}

/// One GitHub organization registration and its own repository policy.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct OrganizationView {
    /// Stable ciproxy registration identity.
    pub id: String,
    /// Stable GitHub organization identity as a string.
    pub github_id: String,
    /// Current GitHub organization name.
    pub login: String,
    /// Current installation identity as a string.
    pub installation_id: String,
    /// Active, suspended, or removed App state.
    pub state: InstallationState,
    /// Owner-removal retention deadline, absent for an intentional take-out.
    pub delete_after: Option<DateTime<Utc>>,
    /// This organization's allowed private repositories.
    pub access: RepositorySelection,
    /// GitHub URL for installation visibility changes.
    pub manage_url: String,
}

/// Current GitHub App state shown by setup and Settings.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InstallationState {
    /// GitHub permits starts.
    Active,
    /// GitHub suspended the App without removing the organization.
    Suspended,
    /// The App was removed.
    Removed,
}

/// Private repository policy, independent for each organization.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum RepositorySelection {
    /// Allow every visible private repository, including future ones.
    AllPrivate,
    /// Allow only the supplied stable GitHub repository identities.
    Selected {
        /// Repository identities as decimal strings.
        ids: Vec<String>,
    },
}

/// One installation-visible repository. Public repositories cannot get a runner.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct RepositoryView {
    /// Stable GitHub repository identity as a string.
    pub id: String,
    /// Owner and repository name returned by GitHub.
    pub full_name: String,
    /// Whether the repository is private and can be selected.
    pub private: bool,
    /// Whether this private repository currently has access.
    pub allowed: bool,
}

/// Label definition whose name never changes after creation.
#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct LabelInput {
    /// Workflow label accepted by GitHub.
    pub name: String,
    /// Requested hardware threads.
    pub vcpus: i32,
    /// Requested memory in decimal GB.
    pub memory_gb: i32,
    /// Free job disk in decimal GB, apart from the full image disk.
    pub disk_gb: i32,
    /// Whether own machines may run this label.
    pub allow_machines: bool,
    /// Whether Boat may run this label.
    pub allow_boat: bool,
    /// What happens when neither destination fits.
    pub when_nothing_fits: NothingFits,
}

/// Waiting policy for a label that currently fits nothing.
#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NothingFits {
    /// Keep the job queued until capacity fits.
    Wait,
    /// Permit the largest available Boat size.
    LargestBoat,
}

/// A stored label with its current machine fit.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct LabelView {
    /// Stable label identity.
    pub id: String,
    /// Current label policy.
    pub label: LabelInput,
    /// Capacity range for connected machines that accept jobs.
    pub fit: LabelFit,
}

/// Computed machine capacity for a label preview.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct LabelFit {
    /// Count of connected, non-draining machines.
    pub eligible_machines: u32,
    /// Machines whose prepared capacity or required root allocation is unavailable.
    pub unavailable_machines: u32,
    /// Count of those machines that fit at least one job.
    pub fitting_machines: u32,
    /// Minimum whole jobs per fitting machine, absent when none fits.
    pub minimum_jobs: Option<String>,
    /// Maximum whole jobs per fitting machine, absent when none fits.
    pub maximum_jobs: Option<String>,
    /// Whether no allowed own machine currently fits this size.
    pub no_machine_fits: bool,
}

/// One workspace-owned organization.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct OrganizationPath {
    /// Stable workspace identity.
    #[param(value_type=Uuid)]
    pub workspace_id: WorkspaceId,
    /// Stable organization registration identity.
    pub organization_id: Uuid,
}

/// One workspace-owned label.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct LabelPath {
    /// Stable workspace identity.
    #[param(value_type=Uuid)]
    pub workspace_id: WorkspaceId,
    /// Stable label identity.
    pub label_id: Uuid,
}

/// First-job progress for the last non-Boat Setup step.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct SetupJob {
    /// Stable job identity.
    pub id: String,
    /// Job name supplied by GitHub.
    pub name: String,
    /// Repository that owns the job.
    pub repository: String,
    /// Current UUID demand phase supplied by GitHub.
    pub phase: SetupPhase,
    /// Workflow file identity, without treating it as a checked job ref.
    pub workflow_ref: String,
    /// GitHub result, absent until completion.
    pub result: Option<String>,
    /// Execution destination, absent until a runner is assigned.
    pub place: Option<SetupPlace>,
    /// GitHub scale-set assignment time, absent when GitHub supplied none.
    pub assigned_at: Option<DateTime<Utc>>,
    /// Runner start time.
    pub started_at: Option<DateTime<Utc>>,
    /// GitHub completion time.
    pub completed_at: Option<DateTime<Utc>>,
}

/// GitHub scale-set demand phases, without invented queue or Actions identities.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SetupPhase {
    /// GitHub offered the job before assigning it to the scale set.
    Available,
    /// GitHub assigned the job to this scale set.
    Assigned,
    /// A runner started this job.
    Started,
    /// GitHub completed or ended this assignment.
    Completed,
}

/// Actual runner placement known from the matching durable runner record.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SetupPlace {
    /// A VM on an own machine.
    Machine,
    /// A ciproxy-created Boat sandbox.
    Boat,
}
