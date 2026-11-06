//! Queries for atomic installation setup and durable scale-set demand.

mod attempts;
mod facts;
mod jobs;
mod lease;
mod lifecycle;
mod registration;
mod repositories;
mod scale_sets;
mod timing;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::github::{
    GithubJob, GithubStore, JobDelivery, OrganizationLease, Repository, RunFacts,
};
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use crate::connection::PgStore;

impl GithubStore for PgStore {
    fn incomplete_github_job_attempts(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<ciproxy_store_interface::github_timing::PendingTiming>> {
        timing::pending(self, workspace_id)
    }
    fn save_github_job_times(
        &self,
        workspace_id: Uuid,
        attempt_id: Uuid,
        times: &ciproxy_store_interface::github_timing::JobTimes,
        observed_at: DateTime<Utc>,
    ) -> Result<()> {
        timing::save(self, workspace_id, attempt_id, times, observed_at)
    }

    fn lock_github_organization(&self, github_id: i64) -> Result<Box<dyn OrganizationLease>> {
        lease::acquire(self, github_id)
    }
    fn github_runner_group(&self, github_id: i64) -> Result<Option<i64>> {
        scale_sets::group(self, github_id)
    }
    fn save_github_scale_set(&self, workspace_id: Uuid, scale_set: &ScaleSet) -> Result<ScaleSet> {
        scale_sets::save(self, workspace_id, scale_set)
    }
    fn github_run_facts(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        run_id: i64,
    ) -> Result<Option<RunFacts>> {
        facts::lookup(self, workspace_id, organization_id, run_id)
    }
    fn pending_initial_admin(&self, workspace_id: Uuid) -> Result<Option<i64>> {
        registration::pending_admin(self, workspace_id)
    }

    fn confirm_initial_admin(&self, workspace_id: Uuid) -> Result<()> {
        registration::confirm_admin(self, workspace_id)
    }

    fn replace_repositories(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        repositories: &[Repository],
    ) -> Result<()> {
        repositories::replace(self, workspace_id, organization_id, repositories)
    }

    fn repositories(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<Repository>> {
        repositories::list(self, workspace_id, organization_id)
    }

    fn save_github_job(
        &self,
        workspace_id: Uuid,
        job: &GithubJob,
        observed_at: DateTime<Utc>,
    ) -> Result<()> {
        jobs::save(self, workspace_id, job, observed_at)
    }

    fn pending_github_jobs(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        scale_set_id: Uuid,
    ) -> Result<Vec<JobDelivery>> {
        jobs::pending(self, workspace_id, organization_id, scale_set_id)
    }

    fn confirm_github_job(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        job_id: Uuid,
        revision: i64,
    ) -> Result<()> {
        jobs::confirm(self, workspace_id, organization_id, job_id, revision)
    }

    fn github_jobs(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<GithubJob>> {
        jobs::list(self, workspace_id, organization_id)
    }

    fn purge_github_jobs(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<usize> {
        jobs::purge(self, workspace_id, at)
    }
}
