//! Atomic account replacement, observation updates, and key retirement.

use chrono::{DateTime, Utc};
use ciproxy_schema::boat::boat_accounts;
use ciproxy_schema::overflow::overflow_accounts;
use ciproxy_schema::settings::settings;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::store::AccountRecord;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::transaction;

pub(super) fn connect(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    account: &AccountRecord,
    limit: i64,
) -> Result<()> {
    let json = serde_json::to_string(&account.state).internal()?;
    transaction(connection, |connection| {
        workspaces::table
            .find(workspace_id)
            .select(workspaces::id)
            .for_update()
            .first::<Uuid>(connection)
            .store()?;
        crate::boat::retire_keys(connection, workspace_id, account.account.connected_at).store()?;
        diesel::update(settings::table.find(workspace_id))
            .set(settings::boat_spend_limit_usd_micros.eq(limit))
            .execute(connection)
            .store()?;
        diesel::insert_into(boat_accounts::table)
            .values((
                boat_accounts::id.eq(account.account.id),
                boat_accounts::workspace_id.eq(workspace_id),
                boat_accounts::account_name.eq(&account.account.account_name),
                boat_accounts::key_ciphertext.eq(account.key.as_bytes()),
                boat_accounts::connected_at.eq(account.account.connected_at),
            ))
            .execute(connection)
            .store()?;
        diesel::insert_into(overflow_accounts::table)
            .values((
                overflow_accounts::key_id.eq(account.account.id),
                overflow_accounts::state_json.eq(json),
            ))
            .execute(connection)
            .store()?;
        Ok(())
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
            boat_accounts::table
                .filter(boat_accounts::workspace_id.eq(workspace_id))
                .find(account.account.id),
        )
        .set(boat_accounts::rejected_at.eq(account.account.rejected_at))
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
        workspaces::table
            .find(workspace_id)
            .select(workspaces::id)
            .for_update()
            .first::<Uuid>(connection)
            .store()?;
        crate::boat::retire_keys(connection, workspace_id, at).store()
    })
}
