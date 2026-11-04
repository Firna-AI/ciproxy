//! Coherent workspace reads across live facts and anonymous start counts.

use chrono::{DateTime, Utc};
use ciproxy_schema::machines::machines;
use ciproxy_schema::metering::metering_start_totals;
use ciproxy_schema::organizations::organizations;
use ciproxy_store_interface::metering::facts::{
    JobPlace, MachineRegistration, UsageJob, WorkspaceUsage,
};
use ciproxy_store_interface::metering::totals::StartTotal;
use ciproxy_store_interface::vm_size::VmSize;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Timestamptz};
use uuid::Uuid;

use super::job_cursors;

#[derive(QueryableByName)]
struct UsageRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    organization_id: Uuid,
    #[diesel(sql_type = Timestamptz)]
    started_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    completed_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Integer)]
    vcpus: i32,
    #[diesel(sql_type = Integer)]
    memory_gb: i32,
    #[diesel(sql_type = Integer)]
    disk_gb: i32,
    #[diesel(sql_type = Nullable<diesel::sql_types::Uuid>)]
    machine_id: Option<Uuid>,
}

pub(super) fn load(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<WorkspaceUsage> {
    let rows = diesel::sql_query(
        "SELECT j.id, j.organization_id, j.started_at, j.completed_at,
                r.vcpus, r.memory_gb, r.disk_gb, r.machine_id
         FROM jobs j JOIN runners r ON r.id = j.runner_id
           AND r.workspace_id = j.workspace_id AND r.organization_id = j.organization_id
         WHERE j.workspace_id = $1 AND j.started_at IS NOT NULL ORDER BY j.id",
    )
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .load::<UsageRow>(connection)?;
    let jobs = rows
        .into_iter()
        .map(|row| UsageJob {
            id: row.id,
            organization_id: row.organization_id,
            started_at: row.started_at,
            completed_at: row.completed_at,
            size: VmSize {
                vcpus: row.vcpus,
                memory_gb: row.memory_gb,
                disk_gb: row.disk_gb,
            },
            place: if row.machine_id.is_some() {
                JobPlace::Machine
            } else {
                JobPlace::Boat
            },
        })
        .collect();
    let machines = machines::table
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
        .collect();
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
        .load::<(DateTime<Utc>, i64)>(connection)?
        .into_iter()
        .map(|(taken_at, starts)| StartTotal { taken_at, starts })
        .collect();
    Ok(WorkspaceUsage {
        jobs,
        machines,
        organizations,
        starts,
        accounted_jobs: job_cursors::load(connection, workspace_id)?,
    })
}
