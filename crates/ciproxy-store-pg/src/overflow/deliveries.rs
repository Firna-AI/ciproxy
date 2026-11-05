//! Durable reservation and atomic provider identity, runner, and cost writes.

use ciproxy_schema::overflow::overflow_starts;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase, StartRecord};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;
use diesel::sql_types::{Binary, Text};
use uuid::Uuid;

use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::transaction;

use super::placements;

pub(super) fn reserve(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    record: &StartRecord,
    launch: &EncryptedLaunch,
    token_hash: &TokenHash,
) -> Result<bool> {
    let json = serde_json::to_string(record).internal()?;
    transaction(connection, |connection| {
        workspaces::table
            .find(workspace_id)
            .select(workspaces::id)
            .for_update()
            .first::<Uuid>(connection)
            .store()?;
        let count = diesel::sql_query(
            "INSERT INTO overflow_starts
                (delivery_id, workspace_id, runner_id, key_id, record_json, launch_ciphertext, token_hash)
             SELECT $1, $2, $3, $4, $5, $6, $7
             WHERE EXISTS (SELECT 1 FROM boat_accounts WHERE id = $4 AND workspace_id = $2
                 AND retired_at IS NULL AND rejected_at IS NULL)
             AND EXISTS (SELECT 1 FROM settings WHERE workspace_id = $2
                 AND overflow_enabled AND boat_spend_limit_usd_micros > 0)
             AND EXISTS (SELECT 1 FROM organizations WHERE id = $8 AND workspace_id = $2
                 AND app_state = 'active')
             AND EXISTS (SELECT 1 FROM scale_sets s JOIN labels l
                 ON l.workspace_id = s.workspace_id AND l.id = s.label_id
                 WHERE s.id = $9 AND s.workspace_id = $2 AND s.organization_id = $8
                 AND s.label_id = $10 AND l.allow_boat)",
        )
        .bind::<diesel::sql_types::Uuid, _>(record.delivery_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Uuid, _>(record.runner_id)
        .bind::<diesel::sql_types::Uuid, _>(record.key_id)
        .bind::<Text, _>(json)
        .bind::<Binary, _>(&launch.0)
        .bind::<Binary, _>(token_hash.as_bytes())
        .bind::<diesel::sql_types::Uuid, _>(record.organization_id)
        .bind::<diesel::sql_types::Uuid, _>(record.scale_set_id)
        .bind::<diesel::sql_types::Uuid, _>(record.label_id)
        .execute(connection).store()?;
        Ok(count == 1)
    })
}

pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    record: &StartRecord,
) -> Result<()> {
    let json = serde_json::to_string(record).internal()?;
    transaction(connection, |connection| {
        let count = diesel::update(
            overflow_starts::table
                .filter(overflow_starts::workspace_id.eq(workspace_id))
                .find(record.delivery_id),
        )
        .set((
            overflow_starts::record_json.eq(json),
            overflow_starts::reported_at.eq(record.reported_at),
            overflow_starts::ends_at.eq(record.ends_at),
        ))
        .execute(connection)
        .store()?;
        require_record(count)?;
        if matches!(
            record.phase,
            StartPhase::Running | StartPhase::Ended | StartPhase::Refused
        ) {
            diesel::update(overflow_starts::table.find(record.delivery_id))
                .set(overflow_starts::launch_ciphertext.eq::<Option<Vec<u8>>>(None))
                .execute(connection)
                .store()?;
        }
        if record.sandbox_id.is_some() {
            placements::save(connection, workspace_id, record)?;
        }
        Ok(())
    })
}
