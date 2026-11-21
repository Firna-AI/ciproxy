//! Monotonic UUID job updates and revisioned durable notifications.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use ciproxy_schema::github::github_jobs;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::{GithubJob, JobDelivery};
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::query_dsl::methods;
use diesel::sql_types::{BigInt, Bool, Jsonb, Nullable, Timestamptz};
use serde_json::Value;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

use super::{attempts, updates};

pub(super) fn save(
    store: &PgStore,
    workspace_id: Uuid,
    job: &GithubJob,
    observed_at: DateTime<Utc>,
) -> Result<()> {
    let body = serde_json::to_value(job).internal()?;
    let phase = updates::phase(job.phase);
    let completion_observed = updates::completion_observation(job.phase, observed_at);
    let merged_body = format!(
        "CASE WHEN excluded.assignment_order = github_jobs.assignment_order THEN {} ELSE excluded.body END",
        updates::merged_body("github_jobs", "excluded.body")
    );
    let completed_observed_at = format!(
        "CASE WHEN excluded.assignment_order = github_jobs.assignment_order THEN {} ELSE excluded.completed_observed_at END",
        updates::completed_observed_at("github_jobs", "excluded.completed_observed_at")
    );
    let condition = format!(
        "github_jobs.workspace_id = excluded.workspace_id AND (excluded.assignment_order > github_jobs.assignment_order OR (excluded.assignment_order = github_jobs.assignment_order AND excluded.phase >= github_jobs.phase)) AND github_jobs.body IS DISTINCT FROM ({merged_body})"
    );
    transaction(&mut *store.connection()?, |connection| {
        methods::FilterDsl::filter(
            diesel::insert_into(github_jobs::table)
                .values((
                    github_jobs::workspace_id.eq(workspace_id),
                    github_jobs::organization_id.eq(job.organization_id),
                    github_jobs::job_id.eq(job.id),
                    github_jobs::scale_set_id.eq(job.scale_set_id),
                    github_jobs::label_id.eq(job.label_id),
                    github_jobs::assigned_at.eq(job.assigned_at),
                    github_jobs::phase.eq(phase),
                    github_jobs::body.eq(&body),
                    github_jobs::completed_observed_at.eq(completion_observed),
                    github_jobs::assignment_order.eq(job
                        .assigned_at
                        .map(|at| at.to_rfc3339_opts(SecondsFormat::Nanos, true))
                        .unwrap_or_default()),
                ))
                .on_conflict((github_jobs::organization_id, github_jobs::job_id))
                .do_update()
                .set((
                    github_jobs::scale_set_id.eq(job.scale_set_id),
                    github_jobs::label_id.eq(job.label_id),
                    github_jobs::assigned_at.eq(job.assigned_at),
                    github_jobs::phase.eq(phase),
                    github_jobs::body.eq(sql::<Jsonb>(&merged_body)),
                    github_jobs::completed_observed_at
                        .eq(sql::<Nullable<Timestamptz>>(&completed_observed_at)),
                    github_jobs::assignment_order.eq(job
                        .assigned_at
                        .map(|at| at.to_rfc3339_opts(SecondsFormat::Nanos, true))
                        .unwrap_or_default()),
                    github_jobs::revision.eq(sql::<BigInt>(updates::NEXT_REVISION)),
                )),
            sql::<Bool>(&condition),
        )
        .execute(connection)
        .store()?;
        attempts::save(connection, workspace_id, job, observed_at)?;
        Ok(())
    })
}

pub(super) fn pending(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
    scale_set_id: Uuid,
) -> Result<Vec<JobDelivery>> {
    github_jobs::table
        .filter(github_jobs::workspace_id.eq(workspace_id))
        .filter(github_jobs::organization_id.eq(organization_id))
        .filter(github_jobs::scale_set_id.eq(scale_set_id))
        .filter(github_jobs::delivered_revision.lt(github_jobs::revision))
        .order((github_jobs::assigned_at, github_jobs::job_id))
        .select((github_jobs::body, github_jobs::revision))
        .load::<(Value, i64)>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|(body, revision)| {
            Ok(JobDelivery {
                job: serde_json::from_value(body).internal()?,
                revision,
            })
        })
        .collect()
}

pub(super) fn confirm(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    revision: i64,
) -> Result<()> {
    diesel::update(
        github_jobs::table
            .filter(github_jobs::workspace_id.eq(workspace_id))
            .filter(github_jobs::organization_id.eq(organization_id))
            .filter(github_jobs::job_id.eq(job_id))
            .filter(github_jobs::revision.eq(revision)),
    )
    .set(github_jobs::delivered_revision.eq(revision))
    .execute(&mut *store.connection()?)
    .store()?;
    Ok(())
}

pub(super) fn list(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
) -> Result<Vec<GithubJob>> {
    github_jobs::table
        .filter(github_jobs::workspace_id.eq(workspace_id))
        .filter(github_jobs::organization_id.eq(organization_id))
        .order((github_jobs::assigned_at, github_jobs::job_id))
        .select(github_jobs::body)
        .load::<Value>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|body| serde_json::from_value(body).internal())
        .collect()
}

pub(super) fn purge(store: &PgStore, workspace_id: Uuid, at: DateTime<Utc>) -> Result<usize> {
    transaction(&mut *store.connection()?, |connection| {
        attempts::purge(connection, workspace_id, at)?;
        diesel::delete(
        github_jobs::table
            .filter(github_jobs::workspace_id.eq(workspace_id))
            .filter(github_jobs::phase.eq(2_i16))
            .filter(
                sql::<Bool>("COALESCE((github_jobs.body->>'completed_at')::timestamptz, github_jobs.completed_observed_at) <= ")
                    .bind::<Timestamptz, _>(at - Duration::days(90)),
            ),
    )
    .execute(connection)
    .store()
    })
}
