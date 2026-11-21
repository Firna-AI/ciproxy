//! Job metadata persistence without job contents or secrets.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::Result;

/// The visible lifecycle state of a GitHub job.
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum JobState {
    /// GitHub has queued the job.
    Queued,
    /// A runner has taken the job.
    Running,
    /// GitHub has completed the job.
    Completed,
}

/// The result reported by GitHub.
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum JobResult {
    /// The job succeeded.
    Success,
    /// The job failed.
    Failure,
    /// A person or GitHub cancelled the job.
    Cancelled,
    /// GitHub skipped the job.
    Skipped,
    /// The job reached its time limit.
    TimedOut,
    /// GitHub requires an action.
    ActionRequired,
    /// The job ended with a neutral result.
    Neutral,
    /// GitHub marked the result stale.
    Stale,
    /// The job failed during startup.
    StartupFailure,
}

/// The content origins of restored cache entries.
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CacheSource {
    /// All restored contents came from this machine's cache.
    Machine,
    /// All restored contents came from the GitHub cache.
    Github,
    /// Restored contents came from both cache stores.
    MachineAndGithub,
}

/// Entry and byte totals measured when the machine cache handled restores.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct MachineHandledCacheCounts {
    /// Total entries restored from all content origins.
    pub restored_entries: i64,
    /// Total bytes restored from all content origins.
    pub restored_bytes: i64,
}

/// Dashboard metadata from trusted control-plane inputs.
///
/// A runner reference preserves its place and actual VM size. Optional ref
/// metadata comes from trusted GitHub job facts supplied by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Job {
    /// The internal job identifier.
    pub id: Uuid,
    /// The GitHub organization whose repository owns the job.
    pub organization_id: Uuid,
    /// The GitHub job identifier used for message deduplication.
    pub github_id: i64,
    /// The label selected by the workflow.
    pub label_id: Uuid,
    /// The scale set that delivered the job message.
    pub scale_set_id: Uuid,
    /// The assigned runner, absent while queued.
    pub runner_id: Option<Uuid>,
    /// The job name.
    pub name: String,
    /// The workflow name.
    pub workflow: String,
    /// The repository reported by GitHub.
    pub repository: String,
    /// The optional ref supplied by the trusted GitHub job-facts service.
    pub git_ref: Option<String>,
    /// The visible lifecycle state.
    pub state: JobState,
    /// The result reported by GitHub.
    pub result: Option<JobResult>,
    /// When GitHub queued the job.
    pub queued_at: DateTime<Utc>,
    /// When its runner took the job.
    pub started_at: Option<DateTime<Utc>>,
    /// When GitHub completed the job.
    pub completed_at: Option<DateTime<Utc>>,
    /// The stored explanation of runner placement.
    pub routing_reason: Option<String>,
    /// Restored content origins, absent for zero entries or unknown metadata.
    pub cache_source: Option<CacheSource>,
    /// Machine-handled measurements, independent of restored content origins.
    ///
    /// Some includes GitHub-only fetches and zero-entry restores.
    /// Direct GitHub use and fallback have no measured count bundle.
    pub machine_handled_cache_counts: Option<MachineHandledCacheCounts>,
}

/// Queries for durable GitHub job messages and retained history.
pub trait JobStore: Send + Sync {
    /// Read live jobs and finished jobs within the caller's tier window.
    ///
    /// Include exact scale-set attempts and omit their mapped Actions duplicate.
    fn dashboard_jobs(
        &self,
        workspace_id: Uuid,
        finished_since: DateTime<Utc>,
    ) -> Result<Vec<crate::job_views::DashboardJob>>;
    /// Persist a new message once per organization and GitHub job identifier.
    ///
    /// Return false for a duplicate without replacing its current metadata.
    fn insert_job(&self, workspace_id: Uuid, job: &Job) -> Result<bool>;
    /// Replace metadata for an existing job in its organization.
    fn update_job(&self, workspace_id: Uuid, job: &Job) -> Result<()>;
    /// Read the retained job metadata of an organization.
    fn jobs(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<Job>>;
    /// Delete completed job metadata after its 90-day retention.
    fn purge_finished_jobs(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<usize>;
}
