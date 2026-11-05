//! Atomic job cursor updates, signed aggregates, and stale-snapshot results.

use chrono::{DateTime, NaiveDate, Utc};
use ciproxy_schema::jobs::jobs;
use ciproxy_schema::metering::metering_job_cursors;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::metering::facts::{JobPlace, UsageJob};
use ciproxy_store_interface::metering::totals::{AccountedJob, DailyDelta, StartTotal};
use ciproxy_store_interface::vm_size::VmSize;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::InternalCause;

use super::aggregates;

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = metering_job_cursors, treat_none_as_null = true)]
struct CursorRow {
    job_id: Uuid,
    workspace_id: Uuid,
    organization_id: Uuid,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    observed_at: DateTime<Utc>,
    vcpus: i32,
    memory_gb: i32,
    disk_gb: i32,
    place: String,
}

impl CursorRow {
    fn from_record(workspace_id: Uuid, record: &AccountedJob) -> Self {
        Self {
            workspace_id,
            job_id: record.job.id,
            organization_id: record.job.organization_id,
            started_at: record.job.started_at,
            completed_at: record.job.completed_at,
            observed_at: record.observed_at,
            vcpus: record.job.size.vcpus,
            memory_gb: record.job.size.memory_gb,
            disk_gb: record.job.size.disk_gb,
            place: match record.job.place {
                JobPlace::Machine => "machine",
                JobPlace::Boat => "boat",
            }
            .to_owned(),
        }
    }

    fn into_record(self) -> QueryResult<AccountedJob> {
        let place = match self.place.as_str() {
            "machine" => JobPlace::Machine,
            "boat" => JobPlace::Boat,
            _ => {
                return Err(diesel::result::Error::DeserializationError(Box::new(
                    InternalCause::InvalidRow,
                )));
            }
        };
        Ok(AccountedJob {
            observed_at: self.observed_at,
            job: UsageJob {
                id: self.job_id,
                organization_id: self.organization_id,
                started_at: self.started_at,
                completed_at: self.completed_at,
                size: VmSize {
                    vcpus: self.vcpus,
                    memory_gb: self.memory_gb,
                    disk_gb: self.disk_gb,
                },
                place,
            },
        })
    }
}

pub(super) fn load(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<Vec<AccountedJob>> {
    metering_job_cursors::table
        .filter(metering_job_cursors::workspace_id.eq(workspace_id))
        .order(metering_job_cursors::job_id)
        .select(CursorRow::as_select())
        .load::<CursorRow>(connection)?
        .into_iter()
        .map(CursorRow::into_record)
        .collect()
}

pub(super) fn apply(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    previous: Option<&AccountedJob>,
    current: &AccountedJob,
    deltas: &[DailyDelta],
    starts: &[StartTotal],
) -> QueryResult<bool> {
    let workspace = workspaces::table
        .find(workspace_id)
        .select(workspaces::metering_retained_from)
        .for_update()
        .first::<Option<NaiveDate>>(connection)
        .optional()?;
    let Some(retained_from) = workspace else {
        return Ok(false);
    };
    let job = jobs::table
        .filter(jobs::workspace_id.eq(workspace_id))
        .find(current.job.id)
        .select(jobs::id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()?;
    if job.is_none() {
        return Ok(false);
    }
    let row = metering_job_cursors::table
        .filter(metering_job_cursors::workspace_id.eq(workspace_id))
        .find(current.job.id)
        .select(CursorRow::as_select())
        .first(connection)
        .optional()?;
    let before = row.map(CursorRow::into_record).transpose()?;
    if before.as_ref() != previous {
        return Ok(false);
    }
    aggregates::apply_usage(connection, workspace_id, deltas, starts, retained_from)?;
    let row = CursorRow::from_record(workspace_id, current);
    diesel::insert_into(metering_job_cursors::table)
        .values(&row)
        .on_conflict(metering_job_cursors::job_id)
        .do_update()
        .set(&row)
        .execute(connection)?;
    Ok(true)
}
