//! Durable reservation and atomic provider identity, runner, and cost writes.

use ciproxy_schema::overflow::overflow_starts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::admission::ReservationOutcome;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase, StartRecord};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;
use diesel::sql_types::{Binary, Text};
use uuid::Uuid;

use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::transaction;

use super::{admission, placements};

pub(super) fn reserve(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    record: &StartRecord,
    launch: &EncryptedLaunch,
    token_hash: &TokenHash,
) -> Result<bool> {
    if record.phase != StartPhase::Unsent || record.dispatch.is_some() {
        return Ok(false);
    }
    let json = serde_json::to_string(record).internal()?;
    transaction(connection, |connection| {
        let outcome = admission::check(connection, workspace_id, record, &json, None)?;
        if outcome != ReservationOutcome::Reserved {
            return Ok(false);
        }
        let count = diesel::sql_query(
            "INSERT INTO overflow_starts
                (delivery_id, workspace_id, runner_id, key_id, record_json, launch_ciphertext, token_hash)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind::<diesel::sql_types::Uuid, _>(record.delivery_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Uuid, _>(record.runner_id)
        .bind::<diesel::sql_types::Uuid, _>(record.key_id)
        .bind::<Text, _>(json)
        .bind::<Binary, _>(&launch.0)
        .bind::<Binary, _>(token_hash.as_bytes())
        .execute(connection).store()?;
        Ok(count == 1)
    })
}

pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    record: &StartRecord,
) -> Result<()> {
    transaction(connection, |connection| {
        let delivery = || {
            overflow_starts::table
                .filter(overflow_starts::workspace_id.eq(workspace_id))
                .find(record.delivery_id)
        };
        let saved = delivery()
            .select(overflow_starts::record_json)
            .for_update()
            .first::<String>(connection)
            .store()?;
        let saved = serde_json::from_str(&saved).internal()?;
        let merged = retain_lifecycle(record, &saved);
        let json = serde_json::to_string(&merged).internal()?;
        let count = diesel::update(delivery())
            .set((
                overflow_starts::record_json.eq(json),
                overflow_starts::reported_at.eq(merged.reported_at),
                overflow_starts::ends_at.eq(merged.ends_at),
            ))
            .execute(connection)
            .store()?;
        require_record(count)?;
        if launch_input_unused(record.phase) || launch_input_unused(merged.phase) {
            diesel::update(overflow_starts::table.find(record.delivery_id))
                .set(overflow_starts::launch_ciphertext.eq::<Option<Vec<u8>>>(None))
                .execute(connection)
                .store()?;
        }
        placements::save(connection, workspace_id, &merged)
    })
}

fn retain_lifecycle(record: &StartRecord, saved: &StartRecord) -> StartRecord {
    let mut merged = record.clone();
    merged.reported_at = saved.reported_at.or(record.reported_at);
    merged.ends_at = saved.ends_at.or(record.ends_at);
    merged.phase = match saved.phase {
        StartPhase::Deleting if merged.ends_at.is_some() => StartPhase::Ended,
        StartPhase::Deleting | StartPhase::Ended => saved.phase,
        _ if saved.ends_at.is_some() => saved.phase,
        _ => record.phase,
    };
    merged
}

fn launch_input_unused(phase: StartPhase) -> bool {
    matches!(
        phase,
        StartPhase::Running | StartPhase::Ended | StartPhase::Refused | StartPhase::CancelledUnsent
    )
}
