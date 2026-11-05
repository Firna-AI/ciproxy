//! Current label queries for pending dispatch preparation.

use ciproxy_schema::labels::labels;
use ciproxy_schema::overflow::overflow_starts;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::error::ResultExt;
use ciproxy_store_interface::overflow::admission::DispatchFacts;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::read_snapshot;
use crate::labels::LabelRow;

use super::authority;

pub(super) fn facts(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<DispatchFacts>> {
    read_snapshot(connection, |connection| {
        if authority::startup_account(connection, workspace_id, delivery_id)?.is_none() {
            return Ok(None);
        }
        let json = overflow_starts::table
            .filter(overflow_starts::workspace_id.eq(workspace_id))
            .find(delivery_id)
            .select(overflow_starts::record_json)
            .first::<String>(connection)
            .store()?;
        let record: StartRecord = serde_json::from_str(&json).internal()?;
        if record.phase != StartPhase::Unsent {
            return Ok(None);
        }
        let label = labels::table
            .filter(labels::workspace_id.eq(workspace_id))
            .find(record.label_id)
            .select(LabelRow::as_select())
            .first::<LabelRow>(connection)
            .store()?
            .into_record()?;
        Ok(Some(DispatchFacts { label }))
    })
}
