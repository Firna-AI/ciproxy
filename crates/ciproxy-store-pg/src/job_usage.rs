//! Saves and reads each report as a set of samples and ordered steps.

mod boat;
mod own;
mod rows;

use chrono::{DateTime, Utc};
use ciproxy_protocol::usage::JobStep;
use ciproxy_schema::job_usage::{job_usage_samples, job_usage_steps};
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::job_usage::{
    BoatSampleWrite, BoatUsageTarget, JobUsageStore, OwnUsageTarget, StoredJobUsage,
};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;
use diesel::query_dsl::methods;
use diesel::sql_types;
use diesel::upsert::excluded;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::{read_snapshot, transaction};

use self::rows::{SampleRow, StepRow};

fn upsert_samples(connection: &mut PgConnection, row: &SampleRow) -> Result<bool> {
    let newly_final =
        excluded(job_usage_samples::is_final).and(job_usage_samples::is_final.eq(false));
    let newer_of_same_kind = excluded(job_usage_samples::is_final)
        .eq(job_usage_samples::is_final)
        .and(excluded(job_usage_samples::received_at).gt(job_usage_samples::received_at));
    let written = methods::FilterDsl::filter(
        diesel::insert_into(job_usage_samples::table)
            .values(row)
            .on_conflict((
                job_usage_samples::workspace_id,
                job_usage_samples::runner_id,
            ))
            .do_update()
            .set((
                job_usage_samples::platform.eq(excluded(job_usage_samples::platform)),
                job_usage_samples::first_point_at_ms
                    .eq(excluded(job_usage_samples::first_point_at_ms)),
                job_usage_samples::point_seconds.eq(excluded(job_usage_samples::point_seconds)),
                job_usage_samples::cpu_milli_vcpu.eq(excluded(job_usage_samples::cpu_milli_vcpu)),
                job_usage_samples::peak_memory_mib.eq(excluded(job_usage_samples::peak_memory_mib)),
                job_usage_samples::oom_kills.eq(excluded(job_usage_samples::oom_kills)),
                job_usage_samples::is_final.eq(excluded(job_usage_samples::is_final)),
                job_usage_samples::received_at.eq(excluded(job_usage_samples::received_at)),
            )),
        newly_final.or(newer_of_same_kind),
    )
    .execute(connection)
    .store()?;
    Ok(written == 1)
}

/// Expire reports that never found matching job metadata.
pub(crate) fn purge_unmatched(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    cutoff: DateTime<Utc>,
) -> Result<()> {
    diesel::sql_query(
        "DELETE FROM job_usage_samples AS sample USING runners AS runner
         WHERE sample.workspace_id = $1 AND sample.received_at <= $2
            AND runner.workspace_id = sample.workspace_id AND runner.id = sample.runner_id
            AND NOT runner_has_job_usage_history(runner)",
    )
    .bind::<sql_types::Uuid, _>(workspace_id)
    .bind::<sql_types::Timestamptz, _>(cutoff)
    .execute(connection)
    .store()?;
    Ok(())
}

fn replace_steps(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    runner_id: Uuid,
    steps: &[JobStep],
) -> Result<()> {
    diesel::delete(
        job_usage_steps::table
            .filter(job_usage_steps::workspace_id.eq(workspace_id))
            .filter(job_usage_steps::runner_id.eq(runner_id)),
    )
    .execute(connection)
    .store()?;
    if !steps.is_empty() {
        let rows = steps
            .iter()
            .enumerate()
            .map(|(position, step)| StepRow::from_step(workspace_id, runner_id, position, step))
            .collect::<Result<Vec<_>>>()?;
        diesel::insert_into(job_usage_steps::table)
            .values(rows)
            .execute(connection)
            .store()?;
    }
    Ok(())
}

fn write_report(
    connection: &mut PgConnection,
    sample: &SampleRow,
    workspace_id: Uuid,
    runner_id: Uuid,
    steps: &[JobStep],
) -> Result<bool> {
    if !upsert_samples(connection, sample)? {
        return Ok(false);
    }
    replace_steps(connection, workspace_id, runner_id, steps)?;
    Ok(true)
}

impl JobUsageStore for PgStore {
    fn own_usage_target(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        runner_id: Uuid,
    ) -> Result<Option<OwnUsageTarget>> {
        own::target(
            &mut *self.connection()?,
            workspace_id,
            machine_id,
            runner_id,
        )
    }

    fn boat_usage_target(
        &self,
        hash: &TokenHash,
        runner_id: Uuid,
    ) -> Result<Option<BoatUsageTarget>> {
        boat::target(&mut *self.connection()?, hash, runner_id)
    }

    fn save_boat_sample(
        &self,
        workspace_id: Uuid,
        delivery_id: Uuid,
        runner_id: Uuid,
        report: &StoredJobUsage,
        previous_at_or_before: DateTime<Utc>,
    ) -> Result<BoatSampleWrite> {
        boat::save(
            &mut *self.connection()?,
            workspace_id,
            delivery_id,
            runner_id,
            report,
            previous_at_or_before,
        )
    }

    fn save_job_usage(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        report: &StoredJobUsage,
    ) -> Result<bool> {
        let sample = SampleRow::from_report(workspace_id, runner_id, report);
        transaction(&mut *self.connection()?, |connection| {
            write_report(
                connection,
                &sample,
                workspace_id,
                runner_id,
                &report.usage.steps,
            )
        })
    }

    fn job_usage(&self, workspace_id: Uuid, runner_id: Uuid) -> Result<Option<StoredJobUsage>> {
        read_snapshot(&mut *self.connection()?, |connection| {
            let sample = job_usage_samples::table
                .find((workspace_id, runner_id))
                .select(SampleRow::as_select())
                .first(connection)
                .optional()
                .store()?;
            let Some(sample) = sample else {
                return Ok(None);
            };
            let steps = job_usage_steps::table
                .filter(job_usage_steps::workspace_id.eq(workspace_id))
                .filter(job_usage_steps::runner_id.eq(runner_id))
                .order(job_usage_steps::position)
                .select(StepRow::as_select())
                .load(connection)
                .store()?
                .into_iter()
                .map(StepRow::into_step)
                .collect();
            Ok(Some(sample.into_report(steps)?))
        })
    }
}
