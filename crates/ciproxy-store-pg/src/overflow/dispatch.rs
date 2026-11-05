//! Atomic unsent-to-dispatch transition with current authority and monthly limits.

use ciproxy_schema::overflow::overflow_starts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

use super::{admission, deliveries};

pub(super) fn begin(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    record: &StartRecord,
    budget: &ReservationBudget,
) -> Result<ReservationOutcome> {
    let json = serde_json::to_string(record).internal()?;
    transaction(connection, |connection| {
        let outcome = admission::check(connection, workspace_id, record, &json, budget)?;
        if outcome != ReservationOutcome::Reserved {
            return Ok(outcome);
        }
        let saved = overflow_starts::table
            .filter(overflow_starts::workspace_id.eq(workspace_id))
            .find(record.delivery_id)
            .select(overflow_starts::record_json)
            .for_update()
            .first::<String>(connection)
            .store()?;
        let pending: StartRecord = serde_json::from_str(&saved).internal()?;
        if pending.phase != StartPhase::Unsent {
            return Ok(ReservationOutcome::NotPermitted);
        }
        let mut expected = pending;
        expected.phase = StartPhase::Creating;
        expected.starts_at = record.starts_at;
        if record.phase != StartPhase::Creating || expected != *record {
            return Ok(ReservationOutcome::NotPermitted);
        }
        deliveries::save(connection, workspace_id, record)?;
        Ok(ReservationOutcome::Reserved)
    })
}
