//! Reject a complete metering projection when required cost facts are missing.

use chrono::{NaiveDate, NaiveTime};
use ciproxy_schema::overflow::overflow_starts;
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;

/// A linear BoatCost needs a counted fee and a fully measured runtime interval.
pub(crate) fn require_complete(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    range: (NaiveDate, NaiveDate),
) -> Result<()> {
    let records = overflow_starts::table
        .filter(overflow_starts::workspace_id.eq(workspace_id))
        .select(overflow_starts::record_json)
        .load::<String>(connection)
        .store()?;
    for json in records {
        let record: StartRecord = serde_json::from_str(&json).internal()?;
        let (from, until) = range;
        let from_time = from.and_time(NaiveTime::MIN).and_utc();
        let relevant = record.starts_at.date_naive() < until
            && (record.ends_at.is_none_or(|end| end > from_time)
                || record.starts_at.date_naive() >= from);
        if relevant
            && !matches!(
                record.phase,
                StartPhase::Unsent | StartPhase::CancelledUnsent | StartPhase::Refused
            )
            && (!record.runtime_complete || record.counted_start_fee_usd().is_none())
        {
            return Err(Error::AccountingIncomplete);
        }
    }
    Ok(())
}
