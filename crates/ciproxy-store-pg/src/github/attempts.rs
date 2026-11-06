//! Durable per-assignment facts preserve starts when one job UUID restarts.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use ciproxy_schema::github_attempts::github_job_attempts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use diesel::prelude::*;
use diesel::sql_types::{self, Jsonb, Nullable, SmallInt, Text, Timestamptz};
use uuid::Uuid;

use crate::error::QueryResultExt;

pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    job: &GithubJob,
    observed_at: DateTime<Utc>,
) -> Result<()> {
    let Some(assigned) = job.assigned_at else {
        return Ok(());
    };
    let phase = match job.phase {
        DemandPhase::Available => -1,
        DemandPhase::Assigned => 0,
        DemandPhase::Started => 1,
        DemandPhase::Completed => 2,
    };
    diesel::sql_query("INSERT INTO github_job_attempts (workspace_id,organization_id,job_id,assignment_order,phase,body,completed_observed_at)
       VALUES ($1,$2,$3,$4,$5,$6,$7)
       ON CONFLICT (workspace_id,organization_id,job_id,assignment_order) DO UPDATE SET
       phase=GREATEST(github_job_attempts.phase,EXCLUDED.phase),
       completed_observed_at=COALESCE(github_job_attempts.completed_observed_at,EXCLUDED.completed_observed_at),
       body=CASE WHEN EXCLUDED.phase>=github_job_attempts.phase THEN
         EXCLUDED.body || jsonb_build_object(
           'started_at',COALESCE(NULLIF(EXCLUDED.body->'started_at','null'::jsonb),github_job_attempts.body->'started_at'),
           'completed_at',COALESCE(NULLIF(EXCLUDED.body->'completed_at','null'::jsonb),github_job_attempts.body->'completed_at'),
           'runner_github_id',COALESCE(NULLIF(EXCLUDED.body->'runner_github_id','null'::jsonb),github_job_attempts.body->'runner_github_id'),
           'actions_job_id',COALESCE(NULLIF(EXCLUDED.body->'actions_job_id','null'::jsonb),github_job_attempts.body->'actions_job_id'))
         ELSE github_job_attempts.body END")
       .bind::<sql_types::Uuid,_>(workspace_id).bind::<sql_types::Uuid,_>(job.organization_id).bind::<sql_types::Uuid,_>(job.id)
       .bind::<Text,_>(assigned.to_rfc3339_opts(SecondsFormat::Nanos,true)).bind::<SmallInt,_>(phase)
       .bind::<Jsonb,_>(serde_json::to_value(job).internal()?).bind::<Nullable<Timestamptz>,_>((job.phase==DemandPhase::Completed).then_some(observed_at)).execute(connection).store()?;
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
