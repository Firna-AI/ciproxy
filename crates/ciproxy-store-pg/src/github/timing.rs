//! Atomic timing patches keep listener facts of each exact assignment.

use chrono::{DateTime, Utc};
use ciproxy_schema::github_attempts::github_job_attempts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::github_timing::{JobTimes, PendingTiming};
use diesel::prelude::*;
use diesel::sql_types::{self, Jsonb, Nullable, SmallInt, Timestamptz};
use serde::Serialize;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

use super::updates;

/// These fields patch the current row without replacing listener metadata.
#[derive(Serialize)]
struct TimingUpdate {
    actions_job_id: i64,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phase: Option<DemandPhase>,
}

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
    let update = TimingUpdate {
        actions_job_id: times.actions_job_id,
        started_at: times.started_at,
        completed_at: times.completed_at,
        phase: times.terminal.then_some(DemandPhase::Completed),
    };
    let body = serde_json::to_value(update).internal()?;
    let phase = if times.terminal {
        DemandPhase::Completed
    } else {
        DemandPhase::Available
    };
    let completion_observed = updates::completion_observation(phase, observed_at);
    let merged_body = updates::merged_body("github_jobs", "github_jobs.body || $3");
    let merged_phase = updates::monotonic_phase("github_jobs", "$4");
    let completed_observed_at = updates::completed_observed_at("github_jobs", "$5");
    let demand_query = format!(
        "UPDATE github_jobs SET body = {merged_body}, phase = {merged_phase},
         completed_observed_at = {completed_observed_at}, revision = {}
         FROM github_job_attempts AS attempt
         WHERE attempt.workspace_id = $1 AND attempt.id = $2
         AND github_jobs.workspace_id = attempt.workspace_id
         AND github_jobs.organization_id = attempt.organization_id
         AND github_jobs.job_id = attempt.job_id
         AND github_jobs.assignment_order = attempt.assignment_order
         AND github_jobs.body IS DISTINCT FROM ({merged_body})",
        updates::NEXT_REVISION
    );
    let merged_body = updates::merged_body("github_job_attempts", "github_job_attempts.body || $3");
    let merged_phase = updates::monotonic_phase("github_job_attempts", "$4");
    let completed_observed_at = updates::completed_observed_at("github_job_attempts", "$5");
    let attempt_query = format!(
        "UPDATE github_job_attempts SET body = {merged_body}, phase = {merged_phase},
         completed_observed_at = {completed_observed_at}
         WHERE workspace_id = $1 AND id = $2"
    );
    transaction(&mut *store.connection()?, |connection| {
        diesel::sql_query(
            "SELECT github_jobs.job_id FROM github_jobs
             INNER JOIN github_job_attempts AS attempt
             ON github_jobs.workspace_id = attempt.workspace_id
             AND github_jobs.organization_id = attempt.organization_id
             AND github_jobs.job_id = attempt.job_id
             WHERE attempt.workspace_id = $1 AND attempt.id = $2
             FOR UPDATE OF github_jobs",
        )
        .bind::<sql_types::Uuid, _>(workspace_id)
        .bind::<sql_types::Uuid, _>(attempt_id)
        .execute(connection)
        .store()?;
        diesel::sql_query(&demand_query)
            .bind::<sql_types::Uuid, _>(workspace_id)
            .bind::<sql_types::Uuid, _>(attempt_id)
            .bind::<Jsonb, _>(&body)
            .bind::<SmallInt, _>(updates::phase(phase))
            .bind::<Nullable<Timestamptz>, _>(completion_observed)
            .execute(connection)
            .store()?;
        diesel::sql_query(&attempt_query)
            .bind::<sql_types::Uuid, _>(workspace_id)
            .bind::<sql_types::Uuid, _>(attempt_id)
            .bind::<Jsonb, _>(&body)
            .bind::<SmallInt, _>(updates::phase(phase))
            .bind::<Nullable<Timestamptz>, _>(completion_observed)
            .execute(connection)
            .store()?;
        Ok(())
    })
}
