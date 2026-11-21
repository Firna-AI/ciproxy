//! Atomic one-use callback claims persisted before sandbox deletion.

use chrono::{DateTime, Utc};
use ciproxy_schema::overflow::overflow_starts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

use super::deliveries;

pub(super) fn accept(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    hash: &TokenHash,
    runner_id: Uuid,
    at: DateTime<Utc>,
) -> Result<Option<StartRecord>> {
    transaction(connection, |connection| {
        let json = overflow_starts::table
            .filter(overflow_starts::workspace_id.eq(workspace_id))
            .filter(overflow_starts::token_hash.eq(hash.as_bytes()))
            .filter(overflow_starts::runner_id.eq(runner_id))
            .filter(overflow_starts::reported_at.is_null())
            .filter(overflow_starts::ends_at.is_null())
            .select(overflow_starts::record_json)
            .for_update()
            .first::<String>(connection)
            .optional()
            .store()?;
        let Some(json) = json else {
            return Ok(None);
        };
        let mut record: StartRecord = serde_json::from_str(&json).internal()?;
        record.reported_at = Some(at);
        record.phase = StartPhase::Deleting;
        deliveries::save(connection, workspace_id, &record)?;
        Ok(Some(record))
    })
}
