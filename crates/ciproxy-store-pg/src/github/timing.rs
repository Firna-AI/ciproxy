//! Pending authoritative timing reads preserve exact assignment bodies.

use chrono::{DateTime, Utc};
use ciproxy_schema::github_attempts::github_job_attempts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use ciproxy_store_interface::github_timing::{JobTimes, PendingTiming};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

use super::jobs;

pub(super) fn pending(store: &PgStore, workspace_id: Uuid) -> Result<Vec<PendingTiming>> {
    github_job_attempts::table
        .filter(github_job_attempts::workspace_id.eq(workspace_id))
        .filter(github_job_attempts::phase.ge(1))
        .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(
            "body->>'started_at' IS NULL OR (phase=2 AND body->>'completed_at' IS NULL)",
        ))
        .order(github_job_attempts::id)
        .select((github_job_attempts::id, github_job_attempts::body))
        .load::<(Uuid, serde_json::Value)>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|(attempt_id, body)| {
            Ok(PendingTiming {
                attempt_id,
                job: serde_json::from_value(body).internal()?,
            })
        })
        .collect()
}

pub(super) fn save(
    store: &PgStore,
    workspace_id: Uuid,
    attempt_id: Uuid,
    times: &JobTimes,
    observed_at: DateTime<Utc>,
) -> Result<()> {
    let body = github_job_attempts::table
        .filter(github_job_attempts::workspace_id.eq(workspace_id))
        .find(attempt_id)
        .select(github_job_attempts::body)
        .first::<serde_json::Value>(&mut *store.connection()?)
        .optional()
        .store()?;
    let Some(body) = body else {
        return Ok(());
    };
    let mut job: GithubJob = serde_json::from_value(body).internal()?;
    job.started_at = times.started_at.or(job.started_at);
    job.completed_at = times.completed_at.or(job.completed_at);
    job.actions_job_id = Some(times.actions_job_id);
    if times.terminal {
        job.phase = DemandPhase::Completed;
    }
    jobs::save(store, workspace_id, &job, observed_at)
}
