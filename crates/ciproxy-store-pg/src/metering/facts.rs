//! Coherent workspace reads across live facts and anonymous start counts.

use crate::exact_size::{convert, json, parse};
use chrono::{DateTime, Utc};
use ciproxy_protocol::resources::VmSize;
use ciproxy_schema::machines::machines;
use ciproxy_schema::metering::metering_start_totals;
use ciproxy_schema::organizations::organizations;
use ciproxy_store_interface::metering::facts::{
    Completion, JobPlace, JobSource, MachineRegistration, UsageJob, WorkspaceUsage,
};
use ciproxy_store_interface::metering::totals::StartTotal;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Timestamptz};
use uuid::Uuid;

use super::job_cursors;

#[derive(QueryableByName)]
struct UsageRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    source: String,
    #[diesel(sql_type = Nullable<diesel::sql_types::Jsonb>)]
    body: Option<serde_json::Value>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    terminal: bool,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    organization_id: Uuid,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    started_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    completed_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Integer)]
    vcpus: i32,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    memory_bytes: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    disk_bytes: i64,
    #[diesel(sql_type = Nullable<diesel::sql_types::Uuid>)]
    machine_id: Option<Uuid>,
}

pub(super) fn load(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<WorkspaceUsage> {
    let rows = diesel::sql_query(
        "SELECT 'actions'::text AS source, j.id, j.organization_id, j.started_at, j.completed_at,
                r.vcpus, r.memory_bytes, r.disk_bytes, r.machine_id, NULL::jsonb AS body, (j.state = 'completed') AS terminal
         FROM jobs j JOIN runners r ON r.id = j.runner_id
           AND r.workspace_id = j.workspace_id AND r.organization_id = j.organization_id
         WHERE j.workspace_id = $1 AND j.started_at IS NOT NULL
         UNION ALL
         SELECT 'scale_set'::text AS source, g.id, g.organization_id,
                NULL::timestamptz AS started_at, NULL::timestamptz AS completed_at,
                r.vcpus, r.memory_bytes, r.disk_bytes, r.machine_id, g.body AS body, (g.phase = 2) AS terminal
         FROM github_job_attempts g JOIN runners r ON r.workspace_id = g.workspace_id
           AND r.organization_id = g.organization_id AND r.scale_set_id = (g.body->>'scale_set_id')::uuid
           AND r.github_id = (g.body->>'runner_github_id')::bigint
         WHERE g.workspace_id = $1 AND g.phase >= 1
           AND g.body->>'started_at' IS NOT NULL
           AND NOT EXISTS (SELECT 1 FROM jobs j WHERE j.workspace_id = g.workspace_id
             AND j.organization_id = g.organization_id
             AND j.github_id = (g.body->>'actions_job_id')::bigint AND j.started_at IS NOT NULL
             AND j.runner_id = r.id)
         ORDER BY id",
    )
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .load::<UsageRow>(connection)?;
    let jobs = rows
        .into_iter()
        .map(|row| {
            let facts = row
                .body
                .map(json::<ciproxy_store_interface::github::GithubJob>)
                .transpose()?;
            let started_at = facts
                .as_ref()
                .and_then(|job| job.started_at)
                .or(row.started_at)
                .ok_or_else(|| {
                    diesel::result::Error::DeserializationError(Box::new(
                        crate::error::InternalCause::InvalidRow,
                    ))
                })?;
            let completed_at = facts
                .as_ref()
                .and_then(|job| job.completed_at)
                .or(row.completed_at);
            let completion = match (completed_at, row.terminal) {
                (Some(at), _) => Completion::Finished(at),
                (None, true) => Completion::TerminalUnknown,
                (None, false) => Completion::Running,
            };
            QueryResult::Ok(UsageJob {
                source: match row.source.as_str() {
                    "scale_set" => JobSource::ScaleSet,
                    _ => JobSource::Actions,
                },
                id: row.id,
                organization_id: row.organization_id,
                started_at,
                completion,
                size: VmSize {
                    vcpu: convert(row.vcpus)?,
                    memory_bytes: convert(row.memory_bytes)?,
                    disk_bytes: convert(row.disk_bytes)?,
                },
                place: if row.machine_id.is_some() {
                    JobPlace::Machine
                } else {
                    JobPlace::Boat
                },
            })
        })
        .collect::<QueryResult<Vec<_>>>()?;
    let machines = registrations(connection, workspace_id)?;
    let organizations = organizations::table
        .filter(organizations::workspace_id.eq(workspace_id))
        .filter(organizations::removed_at.is_null())
        .count()
        .get_result::<i64>(connection)? as u64;
    let starts = metering_start_totals::table
        .filter(metering_start_totals::workspace_id.eq(workspace_id))
        .order(metering_start_totals::taken_at)
        .select((
            metering_start_totals::taken_at,
            metering_start_totals::starts,
        ))
        .load::<(String, i64)>(connection)?
        .into_iter()
        .map(|(taken_at, starts)| {
            QueryResult::Ok(StartTotal {
                taken_at: parse(&taken_at)?,
                starts,
            })
        })
        .collect::<QueryResult<Vec<_>>>()?;
    Ok(WorkspaceUsage {
        aliases: super::aliases::load(connection, workspace_id)?,
        jobs,
        machines,
        organizations,
        starts,
        accounted_jobs: job_cursors::load(connection, workspace_id)?,
    })
}

pub(super) fn registrations(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<Vec<MachineRegistration>> {
    Ok(machines::table
        .filter(machines::workspace_id.eq(workspace_id))
        .order(machines::id)
        .select((machines::id, machines::registered_at, machines::removed_at))
        .load::<(Uuid, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(connection)?
        .into_iter()
        .map(
            |(machine_id, registered_at, removed_at)| MachineRegistration {
                machine_id,
                registered_at,
                removed_at,
            },
        )
        .collect())
}
