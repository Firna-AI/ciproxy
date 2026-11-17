//! Durable per-assignment facts preserve starts when one job UUID restarts.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use ciproxy_schema::github_attempts::github_job_attempts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::GithubJob;
use diesel::prelude::*;
use diesel::sql_types::{self, Jsonb, Nullable, SmallInt, Text, Timestamptz};
use uuid::Uuid;

use crate::error::QueryResultExt;

use super::updates;

pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    job: &GithubJob,
    observed_at: DateTime<Utc>,
) -> Result<()> {
    let Some(assigned) = job.assigned_at else {
        return Ok(());
    };
    let phase = updates::phase(job.phase);
    let merged_body = updates::merged_body("github_job_attempts", "EXCLUDED.body");
    let merged_phase = updates::monotonic_phase("github_job_attempts", "EXCLUDED.phase");
    let completed_observed_at =
        updates::completed_observed_at("github_job_attempts", "EXCLUDED.completed_observed_at");
    diesel::sql_query(format!("INSERT INTO github_job_attempts (workspace_id,organization_id,job_id,assignment_order,phase,body,completed_observed_at)
       VALUES ($1,$2,$3,$4,$5,$6,$7)
       ON CONFLICT (workspace_id,organization_id,job_id,assignment_order) DO UPDATE SET
       phase={merged_phase},
       completed_observed_at={completed_observed_at},
       body=CASE WHEN EXCLUDED.phase>=github_job_attempts.phase THEN
         {merged_body}
         ELSE github_job_attempts.body END"))
       .bind::<sql_types::Uuid,_>(workspace_id).bind::<sql_types::Uuid,_>(job.organization_id).bind::<sql_types::Uuid,_>(job.id)
       .bind::<Text,_>(assigned.to_rfc3339_opts(SecondsFormat::Nanos,true)).bind::<SmallInt,_>(phase)
       .bind::<Jsonb,_>(serde_json::to_value(job).internal()?).bind::<Nullable<Timestamptz>,_>(updates::completion_observation(job.phase, observed_at)).execute(connection).store()?;
    Ok(())
}

pub(super) fn purge(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    at: DateTime<Utc>,
) -> Result<()> {
    diesel::delete(github_job_attempts::table
        .filter(github_job_attempts::workspace_id.eq(workspace_id))
        .filter(github_job_attempts::phase.eq(2_i16))
        .filter(diesel::dsl::sql::<sql_types::Bool>(
            "COALESCE((github_job_attempts.body->>'completed_at')::timestamptz, github_job_attempts.completed_observed_at) <= ")
            .bind::<Timestamptz,_>(at-Duration::days(90))))
        .execute(connection).store()?;
    Ok(())
}
