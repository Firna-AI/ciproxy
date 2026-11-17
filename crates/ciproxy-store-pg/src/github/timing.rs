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

/// A captured runner target proves that the current listener cannot finish this attempt.
const STRANDED_STARTED: &str = concat!(
    "github_job_attempts.phase = 1 AND EXISTS (",
    "SELECT 1 FROM runners r JOIN scale_sets s ON s.id = r.scale_set_id ",
    "AND s.workspace_id = r.workspace_id AND s.organization_id = r.organization_id ",
    "WHERE r.workspace_id = github_job_attempts.workspace_id ",
    "AND r.organization_id = github_job_attempts.organization_id ",
    "AND r.scale_set_id = (github_job_attempts.body->>'scale_set_id')::uuid ",
    "AND r.label_id = (github_job_attempts.body->>'label_id')::uuid ",
    "AND r.github_id = (github_job_attempts.body->>'runner_github_id')::bigint ",
    "AND r.registration_scale_set_id <> s.github_id)",
);

pub(super) fn pending(store: &PgStore, workspace_id: Uuid) -> Result<Vec<PendingTiming>> {
    github_job_attempts::table
        .filter(github_job_attempts::workspace_id.eq(workspace_id))
        .filter(github_job_attempts::phase.ge(1))
        .filter(
            diesel::dsl::sql::<diesel::sql_types::Bool>(
                "body->>'started_at' IS NULL OR (phase=2 AND body->>'completed_at' IS NULL)",
            )
            .or(diesel::dsl::sql::<diesel::sql_types::Bool>(
                STRANDED_STARTED,
            )),
        )
        .order(github_job_attempts::id)
        .select((
            github_job_attempts::id,
            github_job_attempts::body,
            diesel::dsl::sql::<diesel::sql_types::Bool>(STRANDED_STARTED),
        ))
        .load::<(Uuid, serde_json::Value, bool)>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|(attempt_id, body, awaiting_completion)| {
            Ok(PendingTiming {
                attempt_id,
                job: serde_json::from_value(body).internal()?,
                awaiting_completion,
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
    job.result = job.result.or(times.result.clone());
    if times.terminal {
        job.phase = DemandPhase::Completed;
    }
    jobs::save(store, workspace_id, &job, observed_at)
}
