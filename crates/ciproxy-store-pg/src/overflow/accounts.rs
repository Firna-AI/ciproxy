//! Atomic account replacement, observation updates, and key retirement.

use chrono::{DateTime, Utc};
use ciproxy_schema::overflow::{overflow_accounts, overflow_connection_grants};
use ciproxy_schema::overflow_provider_accounts::overflow_provider_accounts;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::connection::ConnectionGrant;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::store::AccountRecord;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::transaction;
use crate::overflow_provider_accounts::{replace_key, retire_keys};
use crate::workspaces::lock_workspace;

pub(super) fn connect(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    grant: ConnectionGrant,
    account: &AccountRecord,
    limit: i64,
) -> Result<bool> {
    let json = serde_json::to_string(&account.state).internal()?;
    transaction(connection, |connection| {
        let workspace = lock_workspace(connection, workspace_id)
            .optional()
            .store()?;
        if workspace.is_none() {
            return Ok(false);
        }
        let consumed = diesel::delete(
            overflow_connection_grants::table
                .find(grant.id)
                .filter(overflow_connection_grants::workspace_id.eq(workspace_id))
                .filter(overflow_connection_grants::key_id.eq(account.account.id)),
        )
        .execute(connection)
        .store()?;
        if consumed != 1 {
            return Ok(false);
        }
        replace_key(
            connection,
            workspace_id,
            &account.account,
            &account.key,
            limit,
        )
        .store()?;
        diesel::insert_into(overflow_accounts::table)
            .values((
                overflow_accounts::key_id.eq(account.account.id),
                overflow_accounts::state_json.eq(json),
            ))
            .execute(connection)
            .store()?;
        Ok(true)
    })
}

pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    account: &AccountRecord,
) -> Result<()> {
    let json = serde_json::to_string(&account.state).internal()?;
    transaction(connection, |connection| {
        let count = diesel::update(
            overflow_provider_accounts::table
                .filter(overflow_provider_accounts::workspace_id.eq(workspace_id))
                .find(account.account.id),
        )
        .set(overflow_provider_accounts::rejected_at.eq(account.account.rejected_at))
        .execute(connection)
        .store()?;
        require_record(count)?;
        diesel::insert_into(overflow_accounts::table)
            .values((
                overflow_accounts::key_id.eq(account.account.id),
                overflow_accounts::state_json.eq(&json),
            ))
            .on_conflict(overflow_accounts::key_id)
            .do_update()
            .set(overflow_accounts::state_json.eq(&json))
            .execute(connection)
            .store()?;
        Ok(())
    })
}

pub(super) fn retire(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    at: DateTime<Utc>,
) -> Result<()> {
    transaction(connection, |connection| {
        lock_workspace(connection, workspace_id).store()?;
        retire_keys(connection, workspace_id, Some(OverflowProvider::Boat), at).store()
    })
}
