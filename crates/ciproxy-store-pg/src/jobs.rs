//! Durable job metadata, idempotent inserts, and retention queries.

use chrono::{DateTime, Duration, Utc};
use ciproxy_schema::jobs::jobs;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::jobs::{Job, JobStore, MachineHandledCacheCounts};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = jobs, treat_none_as_null = true)]
struct JobRow {
    id: Uuid,
    workspace_id: Uuid,
    organization_id: Uuid,
    github_id: i64,
    label_id: Uuid,
    scale_set_id: Uuid,
    runner_id: Option<Uuid>,
    name: String,
    workflow: String,
    repository: String,
    git_ref: Option<String>,
    state: String,
    result: Option<String>,
    queued_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    routing_reason: Option<String>,
    cache_source: Option<String>,
    cache_restores: Option<i64>,
    cache_restored_bytes: Option<i64>,
}

impl JobRow {
    fn from_record(workspace_id: Uuid, job: &Job) -> Self {
        let state: &str = job.state.into();
        let (cache_restores, cache_restored_bytes) = match job.machine_handled_cache_counts {
            Some(counts) => (Some(counts.restored_entries), Some(counts.restored_bytes)),
            None => (None, None),
        };
        Self {
            id: job.id,
            workspace_id,
            organization_id: job.organization_id,
            github_id: job.github_id,
            label_id: job.label_id,
            scale_set_id: job.scale_set_id,
            runner_id: job.runner_id,
            name: job.name.clone(),
            workflow: job.workflow.clone(),
            repository: job.repository.clone(),
            git_ref: job.git_ref.clone(),
            state: state.to_owned(),
            result: job.result.map(|result| {
                let value: &str = result.into();
                value.to_owned()
            }),
            queued_at: job.queued_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            routing_reason: job.routing_reason.clone(),
            cache_source: job.cache_source.map(|source| {
                let value: &str = source.into();
                value.to_owned()
            }),
            cache_restores,
            cache_restored_bytes,
        }
    }

    fn into_record(self) -> Result<Job> {
        Ok(Job {
            id: self.id,
            organization_id: self.organization_id,
            github_id: self.github_id,
            label_id: self.label_id,
            scale_set_id: self.scale_set_id,
            runner_id: self.runner_id,
            name: self.name,
            workflow: self.workflow,
            repository: self.repository,
            git_ref: self.git_ref,
            state: self.state.parse().internal()?,
            result: self
                .result
                .map(|value| value.parse().internal())
                .transpose()?,
            queued_at: self.queued_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            routing_reason: self.routing_reason,
            cache_source: self
                .cache_source
                .map(|value| value.parse().internal())
                .transpose()?,
            machine_handled_cache_counts: self.cache_restores.zip(self.cache_restored_bytes).map(
                |(restored_entries, restored_bytes)| MachineHandledCacheCounts {
                    restored_entries,
                    restored_bytes,
                },
            ),
        })
    }
}

impl JobStore for PgStore {
    fn dashboard_jobs(
        &self,
        workspace_id: Uuid,
        finished_since: DateTime<Utc>,
    ) -> Result<Vec<ciproxy_store_interface::job_views::DashboardJob>> {
        crate::job_views::load(self, workspace_id, finished_since)
    }
    fn insert_job(&self, workspace_id: Uuid, job: &Job) -> Result<bool> {
        let count = diesel::insert_into(jobs::table)
            .values(JobRow::from_record(workspace_id, job))
            .on_conflict((jobs::organization_id, jobs::github_id))
            .do_nothing()
            .execute(&mut *self.connection()?)
            .store()?;
        Ok(count == 1)
    }

    fn update_job(&self, workspace_id: Uuid, job: &Job) -> Result<()> {
        let row = JobRow::from_record(workspace_id, job);
        let count = diesel::update(
            jobs::table
                .filter(jobs::workspace_id.eq(workspace_id))
                .filter(jobs::organization_id.eq(job.organization_id))
                .find(job.id),
        )
        .set(&row)
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }

    fn jobs(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<Job>> {
        jobs::table
            .filter(jobs::workspace_id.eq(workspace_id))
            .filter(jobs::organization_id.eq(organization_id))
            .order(jobs::queued_at)
            .select(JobRow::as_select())
            .load(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(JobRow::into_record)
            .collect()
    }

    fn purge_finished_jobs(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<usize> {
        diesel::delete(
            jobs::table
                .filter(jobs::workspace_id.eq(workspace_id))
                .filter(jobs::state.eq("completed"))
                .filter(jobs::completed_at.le(at - Duration::days(90))),
        )
        .execute(&mut *self.connection()?)
        .store()
    }
}
