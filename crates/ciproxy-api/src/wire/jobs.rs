//! Jobs live, history, and detail contracts from trusted metadata.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::access::workspace::WorkspaceId;

use super::job_fit::WaitingLabelFit;
use super::job_overflow::JobOverflow;
use super::pages::PageBanner;

/// One job detail path, opened only after current workspace access.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct JobPath {
    /// Opaque workspace candidate.
    #[param(value_type=Uuid)]
    pub workspace_id: WorkspaceId,
    /// Stable job or exact attempt identity.
    pub job_id: Uuid,
}

/// History filters applied within the current plan's visible window.
#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct JobHistoryQuery {
    /// Case-insensitive job, workflow, repository, or organization search.
    pub search: Option<String>,
    /// Execution place, absent for all places.
    #[param(inline)]
    pub place: Option<HistoryPlace>,
    /// First included UTC date, absent for the full plan window.
    pub from: Option<NaiveDate>,
    /// First excluded UTC date.
    pub until: Option<NaiveDate>,
}

/// Execution place filter.
#[derive(Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HistoryPlace {
    /// A customer's own machine.
    Machine,
    /// A Boat sandbox.
    Boat,
}

/// The visible lifecycle, including terminal jobs with unknown finish times.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// No runner has taken the job.
    Queued,
    /// A runner has taken the job.
    Running,
    /// GitHub reports completion.
    Completed,
}

/// Actual runner placement, absent until a runner is identified.
#[derive(Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JobLocation {
    /// A VM on one own machine.
    Machine {
        /// Registered machine identity.
        id: String,
        /// Retained machine hostname.
        name: Option<String>,
        /// First assigned slot, when known.
        first_slot: Option<i32>,
    },
    /// A sandbox on Boat.
    Boat {
        /// Actual billed Boat size name.
        size: String,
    },
}

/// Actual byte-precise VM dimensions, distinct from requested free job disk.
#[derive(Serialize, ToSchema)]
pub struct JobVmSize {
    /// Actual virtual CPUs.
    pub vcpus: u32,
    /// Actual memory bytes as a decimal string.
    pub memory_bytes: String,
    /// Actual root disk bytes as a decimal string.
    pub disk_bytes: String,
}

/// Requested label dimensions.
#[derive(Serialize, ToSchema)]
pub struct JobLabel {
    /// Retained label name.
    pub name: String,
    /// Requested virtual CPUs.
    pub vcpus: i32,
    /// Requested memory in decimal GB.
    pub memory_gb: i32,
    /// Requested free job disk in decimal GB.
    pub disk_gb: i32,
}

/// Restored content origins, independent of whether counts were measured.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobCacheSource {
    /// Own-machine cache contents only.
    Machine,
    /// GitHub cache contents only.
    Github,
    /// Both content stores.
    MachineAndGithub,
}

/// Measurements made only by the machine cache handling a restore.
#[derive(Serialize, ToSchema)]
pub struct JobCacheCounts {
    /// Restored entry count as a decimal string, including zero.
    pub restored_entries: String,
    /// Restored bytes as a decimal string, including zero.
    pub restored_bytes: String,
}

/// Credential-free trusted metadata for one visible job.
#[derive(Serialize, ToSchema)]
pub struct JobView {
    /// Stable job or exact attempt identity.
    pub id: String,
    /// Owning organization identity.
    pub organization_id: String,
    /// Owning organization login.
    pub organization: String,
    /// Trusted job name.
    pub name: String,
    /// Trusted workflow name or file identity.
    pub workflow: String,
    /// Trusted owner and repository.
    pub repository: String,
    /// Trusted full ref, when supplied by the source.
    pub git_ref: Option<String>,
    /// GitHub's branch or tag name, without an invented namespace.
    pub head_branch: Option<String>,
    /// Lifecycle state.
    pub status: JobStatus,
    /// Verbatim result from its trusted source, without vocabulary normalization.
    /// REST examples are success, failure, and cancelled.
    /// Scale-set examples are succeeded, failed, and canceled.
    pub result: Option<String>,
    /// Queue time supplied by GitHub.
    pub queued_at: Option<DateTime<Utc>>,
    /// Runner-taken time supplied by GitHub.
    pub started_at: Option<DateTime<Utc>>,
    /// Actual finish supplied by GitHub, absent for terminal unknown timing.
    pub completed_at: Option<DateTime<Utc>>,
    /// True for a job taken by an identified ciproxy runner, even with unknown time.
    pub counts_as_start: bool,
    /// Wait in exact decimal seconds when both boundaries are known.
    pub waited_seconds: Option<String>,
    /// Runtime in exact decimal seconds, absent for unknown terminal finish.
    pub runtime_seconds: Option<String>,
    /// Requested label and free job disk.
    pub label: JobLabel,
    /// Actual execution place.
    pub location: Option<JobLocation>,
    /// Actual runner resources.
    pub actual_size: Option<JobVmSize>,
    /// Saved routing explanation when supplied.
    pub routing_reason: Option<String>,
    /// GitHub page that holds logs, absent when no stable URL is known.
    pub github_url: Option<String>,
    /// Restored content origins, absent for zero or unknown restores.
    pub cache_source: Option<JobCacheSource>,
    /// Machine-handled measurements, absent for direct GitHub and fallback.
    pub cache_counts: Option<JobCacheCounts>,
}

/// The live queue with explicit unknown waiting visibility.
#[derive(Serialize, ToSchema)]
pub struct JobsLivePage {
    /// Current resource fit for each exact label with visible waiting jobs.
    pub waiting_labels: Vec<WaitingLabelFit>,
    /// Running jobs across all workspace organizations.
    pub running: Vec<JobView>,
    /// Visible waiting jobs with no machine assigned.
    pub waiting: Vec<JobView>,
    /// Complete current-member waiting count, absent during current suspension.
    pub waiting_count: Option<String>,
    /// Own slots with jobs, absent when report data is unavailable.
    pub own_slots_used: Option<String>,
    /// Available own slot capacity, absent when it is unknown.
    pub own_slots_total: Option<String>,
    /// Number of running Boat jobs.
    pub on_boat: String,
    /// Actual ready own-machine runners.
    pub warm_runners: String,
    /// Actual starting runners and their target places.
    pub starting_runners: Vec<StartingRunner>,
    /// Current suspensions and retained owner removals, excluding intentional take-outs.
    pub visibility: Vec<JobVisibility>,
    /// Workspace overflow policy that explains queue wait.
    pub overflow_enabled: bool,
    /// Saved overflow wait in seconds.
    pub overflow_wait_seconds: u32,
    /// Current Boat refusal and spend facts, absent when their source is unavailable.
    pub overflow: Option<JobOverflow>,
    /// Current caller may change labels or overflow settings.
    pub can_manage: bool,
    /// One page banner.
    pub banner: Option<PageBanner>,
}

/// One runner that the runtime is preparing, without registration credentials.
#[derive(Serialize, ToSchema)]
pub struct StartingRunner {
    /// Stable runner identity.
    pub id: String,
    /// Label it will serve.
    pub label_id: String,
    /// Registered own-machine identity, absent for Boat.
    pub machine_id: Option<String>,
    /// True for a Boat sandbox.
    pub boat: bool,
}

/// Current suspension or retained owner removal, without changing current membership.
#[derive(Serialize, ToSchema)]
pub struct JobVisibility {
    /// Organization login.
    pub organization: String,
    /// True while GitHub suspended the App.
    pub suspended: bool,
    /// True after the App was removed.
    pub removed: bool,
    /// Current installation settings, or the configured App install URL after removal.
    pub github_url: String,
}

/// Finished jobs filtered within the plan window, without deleting older data.
#[derive(Serialize, ToSchema)]
pub struct JobsHistoryPage {
    /// Finished jobs that match the filters.
    pub jobs: Vec<JobView>,
    /// Current plan's finished-job window.
    pub history_days: u32,
    /// One page banner.
    pub banner: Option<PageBanner>,
}

/// One authorized visible job with the shared banner.
#[derive(Serialize, ToSchema)]
pub struct JobDetailPage {
    /// Trusted job metadata.
    pub job: JobView,
    /// One page banner.
    pub banner: Option<PageBanner>,
}
