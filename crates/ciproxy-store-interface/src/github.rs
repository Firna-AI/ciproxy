//! Durable GitHub scale-set demand and installation repository snapshots.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::scale_sets::ScaleSet;

/// The repository metadata visible to an App installation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Repository {
    /// The stable GitHub repository identifier.
    pub id: i64,
    /// The owner and repository name returned by GitHub.
    pub full_name: String,
    /// Whether GitHub identifies this repository as private.
    pub private: bool,
    /// The default branch returned by GitHub.
    pub default_branch: String,
}

/// A repository identity from the trusted workflow-run API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RepositoryIdentity {
    /// The stable GitHub repository identifier.
    pub id: i64,
    /// The owner and repository name.
    pub full_name: String,
}

/// Verbatim metadata from GitHub's workflow-run API.
///
/// head_branch is not a full ref. No caller may treat it as one.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunFacts {
    /// The numeric workflow run identifier.
    pub id: i64,
    /// The repository that owns the workflow run.
    pub repository: RepositoryIdentity,
    /// The event returned by the API, independent of workflow file location.
    pub event: String,
    /// The API's branch or tag name, which has no ref namespace.
    pub head_branch: Option<String>,
    /// The commit returned by the API.
    pub head_sha: String,
    /// The source repository, absent if GitHub has no surviving repository.
    pub head_repository: Option<RepositoryIdentity>,
}

/// The monotonic phase within one assignment of a scale-set job.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DemandPhase {
    /// GitHub offered a job that has not yet been assigned to this scale set.
    Available,
    /// GitHub assigned the job to the scale set.
    Assigned,
    /// A runner started the assigned job.
    Started,
    /// GitHub ended the assignment or completed the job.
    Completed,
}

/// One durable demand record keyed by the UUID jobId from scale-set messages.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GithubJob {
    /// The UUID supplied in jobId, independent of numeric Actions job IDs.
    pub id: Uuid,
    /// The owning organization registration.
    pub organization_id: Uuid,
    /// The scale set that delivered the message.
    pub scale_set_id: Uuid,
    /// The shared workspace label.
    pub label_id: Uuid,
    /// The numeric Actions history identity, when a trusted API supplies it.
    pub actions_job_id: Option<i64>,
    /// The request identifier, which GitHub currently sends as zero.
    pub request_id: i64,
    /// The numeric workflow run identifier.
    pub run_id: i64,
    /// The repository from the scale-set message.
    pub repository: String,
    /// The job name from the message.
    pub name: String,
    /// The workflow file identity, never a job ref.
    pub workflow_ref: String,
    /// The latest observed phase of this assignment.
    pub phase: DemandPhase,
    /// The GitHub result when the assignment completed.
    pub result: Option<String>,
    /// scaleSetAssignTime, used as the start of queue waiting.
    pub assigned_at: Option<DateTime<Utc>>,
    /// runnerAssignTime, when present.
    pub started_at: Option<DateTime<Utc>>,
    /// finishTime, when present.
    pub completed_at: Option<DateTime<Utc>>,
    /// The registered runner identifier from a started/completed message.
    pub runner_github_id: Option<i64>,
    /// Trusted API metadata. It contains no invented full ref.
    pub facts: RunFacts,
}

/// A durable pending notification and its optimistic delivery revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobDelivery {
    /// The saved demand and trusted API facts.
    pub job: GithubJob,
    /// The revision that the consumer must confirm after delivery.
    pub revision: i64,
}

/// An owned, waiting organization mutation lease on a dedicated connection.
pub trait OrganizationLease: Send {
    /// Fail when the database session no longer owns the lease.
    fn check_connection(&mut self) -> Result<()>;
}

/// Queries needed only by the hosted GitHub integration.
pub trait GithubStore: Send + Sync {
    /// Read attempts missing actual start or terminal finish timestamps.
    fn incomplete_github_job_attempts(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::github_timing::PendingTiming>>;
    /// Persist only real Actions API timing for the exact retained attempt.
    fn save_github_job_times(
        &self,
        workspace_id: Uuid,
        attempt_id: Uuid,
        times: &crate::github_timing::JobTimes,
        observed_at: DateTime<Utc>,
    ) -> Result<()>;
    /// Wait for exclusive mutation authority over a full GitHub organization ID.
    ///
    /// This uses the two-key advisory namespace, separate from scale-set locks.
    fn lock_github_organization(&self, github_id: i64) -> Result<Box<dyn OrganizationLease>>;
    /// Find the managed group from current or retained registrations of GitHub.
    fn github_runner_group(&self, github_id: i64) -> Result<Option<i64>>;
    /// Save remote identities while retaining the internal UUID and lock key.
    fn save_github_scale_set(&self, workspace_id: Uuid, scale_set: &ScaleSet) -> Result<ScaleSet>;
    /// Reuse trusted facts of a known run within the same tenant and registration.
    fn github_run_facts(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        run_id: i64,
    ) -> Result<Option<RunFacts>>;
    /// Read the first installer's pending role delivery, saved with its workspace.
    fn pending_initial_admin(&self, workspace_id: Uuid) -> Result<Option<i64>>;
    /// Confirm an idempotent initial Admin delivery to the role service.
    fn confirm_initial_admin(&self, workspace_id: Uuid) -> Result<()>;
    /// Atomically replace the installation's visible repository snapshot.
    fn replace_repositories(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        repositories: &[Repository],
    ) -> Result<()>;
    /// Read repository visibility and default branches for one organization.
    fn repositories(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<Repository>>;
    /// Atomically save a job update and a pending notification.
    ///
    /// Replays never move an assignment backwards. A later assignment may
    /// restart the same job. Identical redeliveries keep their revision.
    /// observed_at is the trusted control-plane observation time. The store
    /// keeps the first completion observation without changing GitHub times.
    fn save_github_job(
        &self,
        workspace_id: Uuid,
        job: &GithubJob,
        observed_at: DateTime<Utc>,
    ) -> Result<()>;
    /// Read pending durable notifications of one scale set.
    fn pending_github_jobs(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        scale_set_id: Uuid,
    ) -> Result<Vec<JobDelivery>>;
    /// Confirm one delivered revision without consuming a newer update.
    fn confirm_github_job(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        job_id: Uuid,
        revision: i64,
    ) -> Result<()>;
    /// Read durable demand for a later snapshot or history service.
    fn github_jobs(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<GithubJob>>;
    /// Delete completed demand after 90 days, including pending deliveries.
    ///
    /// Use the first completion observation when GitHub omitted finishTime.
    fn purge_github_jobs(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<usize>;
}
