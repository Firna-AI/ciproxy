//! Atomic job cursor updates, signed aggregates, and stale-snapshot results.

use crate::exact_size::{encode, json, parse};
use chrono::{NaiveDate, SecondsFormat};
use ciproxy_schema::github_attempts::github_job_attempts;
use ciproxy_schema::jobs::jobs;
use ciproxy_schema::metering::metering_job_cursors;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::metering::facts::JobSource;
use ciproxy_store_interface::metering::totals::{AccountedJob, DailyDelta, StartTotal};
use diesel::prelude::*;
use uuid::Uuid;

use super::aggregates;

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = metering_job_cursors, treat_none_as_null = true)]
struct CursorRow {
    job_id: Uuid,
    workspace_id: Uuid,
    organization_id: Uuid,
    observed_at: String,
    source: String,
    facts: serde_json::Value,
}

impl CursorRow {
    fn from_record(workspace_id: Uuid, record: &AccountedJob) -> QueryResult<Self> {
        Ok(Self {
            job_id: record.job.id,
            workspace_id,
            organization_id: record.job.organization_id,
            observed_at: record
                .observed_at
                .to_rfc3339_opts(SecondsFormat::Nanos, true),
            source: match record.job.source {
                JobSource::Actions => "actions",
                JobSource::ScaleSet => "scale_set",
            }
            .to_owned(),
            facts: encode(&record.job)?,
        })
    }
    fn into_record(self) -> QueryResult<AccountedJob> {
        Ok(AccountedJob {
            job: json(self.facts)?,
            observed_at: parse(&self.observed_at)?,
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
    let job = match current.job.source {
        JobSource::Actions => jobs::table
            .filter(jobs::workspace_id.eq(workspace_id))
            .filter(jobs::organization_id.eq(current.job.organization_id))
            .find(current.job.id)
            .select(jobs::id)
            .for_update()
            .first::<Uuid>(connection)
            .optional()?,
        JobSource::ScaleSet => github_job_attempts::table
            .filter(github_job_attempts::workspace_id.eq(workspace_id))
            .filter(github_job_attempts::organization_id.eq(current.job.organization_id))
            .filter(github_job_attempts::id.eq(current.job.id))
            .select(github_job_attempts::id)
            .for_update()
            .first::<Uuid>(connection)
            .optional()?,
    };
    if job.is_none() {
        return Ok(false);
    }
    let row = metering_job_cursors::table
        .filter(metering_job_cursors::workspace_id.eq(workspace_id))
        .filter(metering_job_cursors::organization_id.eq(current.job.organization_id))
        .filter(metering_job_cursors::source.eq(match current.job.source {
            JobSource::Actions => "actions",
            JobSource::ScaleSet => "scale_set",
        }))
        .filter(metering_job_cursors::job_id.eq(current.job.id))
        .select(CursorRow::as_select())
        .first(connection)
        .optional()?;
    let before = row.map(CursorRow::into_record).transpose()?;
    if before.as_ref() != previous {
        return Ok(false);
    }
    aggregates::apply_usage(connection, workspace_id, deltas, starts, retained_from)?;
    let row = CursorRow::from_record(workspace_id, current)?;
    diesel::insert_into(metering_job_cursors::table)
        .values(&row)
        .on_conflict((
            metering_job_cursors::workspace_id,
            metering_job_cursors::organization_id,
            metering_job_cursors::source,
            metering_job_cursors::job_id,
        ))
        .do_update()
        .set(&row)
        .execute(connection)?;
    Ok(true)
}

pub(super) fn retire(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    previous: &AccountedJob,
    deltas: &[DailyDelta],
    starts: &[StartTotal],
) -> QueryResult<bool> {
    let cutoff = workspaces::table
        .find(workspace_id)
        .select(workspaces::metering_retained_from)
        .for_update()
        .first::<Option<NaiveDate>>(connection)?;
    if previous.job.source != JobSource::ScaleSet
        || !super::aliases::exists(
            connection,
            workspace_id,
            previous.job.organization_id,
            previous.job.id,
        )?
    {
        return Ok(false);
    }
    let target = metering_job_cursors::table
        .filter(metering_job_cursors::workspace_id.eq(workspace_id))
        .filter(metering_job_cursors::organization_id.eq(previous.job.organization_id))
        .filter(metering_job_cursors::source.eq("scale_set"))
        .filter(metering_job_cursors::job_id.eq(previous.job.id));
    let row = target
        .select(CursorRow::as_select())
        .first::<CursorRow>(connection)
        .optional()?;
    if row.map(CursorRow::into_record).transpose()?.as_ref() != Some(previous) {
        return Ok(false);
    }
    aggregates::apply_usage(connection, workspace_id, deltas, starts, cutoff)?;
    diesel::delete(target).execute(connection)?;
    Ok(true)
}
