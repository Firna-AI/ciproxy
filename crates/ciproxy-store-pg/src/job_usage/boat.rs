//! Boat callback lookup and sample writes fenced by the saved delivery row.

use chrono::{DateTime, Utc};
use ciproxy_schema::job_usage::job_usage_samples;
use ciproxy_schema::overflow::overflow_starts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::job_usage::{BoatSampleWrite, BoatUsageTarget, StoredJobUsage};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

use super::{SampleRow, write_report};

/// Resolve the token and runner together, including after the recorded end.
pub(super) fn target(
    connection: &mut PgConnection,
    hash: &TokenHash,
    runner_id: Uuid,
) -> Result<Option<BoatUsageTarget>> {
    let target = overflow_starts::table
        .filter(overflow_starts::token_hash.eq(hash.as_bytes()))
        .filter(overflow_starts::runner_id.eq(runner_id))
        .select((overflow_starts::workspace_id, overflow_starts::record_json))
        .first::<(Uuid, String)>(connection)
        .optional()
        .store()?;
    target
        .map(|(workspace_id, json)| {
            Ok(BoatUsageTarget {
                workspace_id,
                record: serde_json::from_str(&json).internal()?,
            })
        })
        .transpose()
}

/// Save only if the delivery is live and the previous receipt permits the write.
pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    delivery_id: Uuid,
    runner_id: Uuid,
    report: &StoredJobUsage,
    previous_at_or_before: DateTime<Utc>,
) -> Result<BoatSampleWrite> {
    let sample = SampleRow::from_sample(workspace_id, runner_id, report);
    transaction(connection, |connection| {
        let live = overflow_starts::table
            .find(delivery_id)
            .filter(overflow_starts::workspace_id.eq(workspace_id))
            .filter(overflow_starts::runner_id.eq(runner_id))
            .filter(overflow_starts::reported_at.is_null())
            .filter(overflow_starts::ends_at.is_null())
            .select(overflow_starts::delivery_id)
            .for_update()
            .first::<Uuid>(connection)
            .optional()
            .store()?;
        if live.is_none() {
            return Ok(BoatSampleWrite::Closed);
        }
        if let Some((is_final, received_at)) = job_usage_samples::table
            .find((workspace_id, runner_id))
            .select((job_usage_samples::is_final, job_usage_samples::received_at))
            .first::<(bool, DateTime<Utc>)>(connection)
            .optional()
            .store()?
        {
            if is_final {
                return Ok(BoatSampleWrite::Closed);
            }
            if received_at > previous_at_or_before {
                return Ok(BoatSampleWrite::TooSoon);
            }
        }
        if write_report(
            connection,
            &sample,
            workspace_id,
            runner_id,
            &report.usage.steps,
        )? {
            Ok(BoatSampleWrite::Saved)
        } else {
            Ok(BoatSampleWrite::Closed)
        }
    })
}
