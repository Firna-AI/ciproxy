//! Ciphertext-only overflow key rotation, retention, and erasure queries.

use chrono::{DateTime, Utc};
use ciproxy_schema::overflow_provider_accounts::overflow_provider_accounts;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow_provider_accounts::{
    OverflowProviderAccount, OverflowProviderAccountStore,
};
use ciproxy_store_interface::secrets::EncryptedOverflowKey;
use diesel::prelude::*;
use diesel::sql_types::{self, Nullable, Text, Timestamptz};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::overflow_provider::DbOverflowProvider;
use crate::overflow_provider_settings::{offer, save_limit};
use crate::workspaces::lock_workspace;

#[derive(Queryable, Selectable)]
#[diesel(table_name = overflow_provider_accounts)]
struct AccountRow {
    id: Uuid,
    provider: DbOverflowProvider,
    account_name: String,
    connected_at: DateTime<Utc>,
    retired_at: Option<DateTime<Utc>>,
    rejected_at: Option<DateTime<Utc>>,
}

impl From<AccountRow> for OverflowProviderAccount {
    fn from(row: AccountRow) -> Self {
        Self {
            id: row.id,
            provider: row.provider.0,
            account_name: row.account_name,
            connected_at: row.connected_at,
            retired_at: row.retired_at,
            rejected_at: row.rejected_at,
        }
    }
}

impl OverflowProviderAccountStore for PgStore {
    fn connect_overflow_provider(
        &self,
        workspace_id: Uuid,
        account: &OverflowProviderAccount,
        key: &EncryptedOverflowKey,
        spend_limit_usd_micros: i64,
    ) -> Result<()> {
        self.connection()?
            .transaction(|connection| {
                lock_workspace(connection, workspace_id)?;
                replace_key(
                    connection,
                    workspace_id,
                    account,
                    key,
                    spend_limit_usd_micros,
                )
            })
            .store()
    }

    fn overflow_provider_accounts(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OverflowProviderAccount>> {
        Ok(overflow_provider_accounts::table
            .filter(overflow_provider_accounts::workspace_id.eq(workspace_id))
            .order(overflow_provider_accounts::connected_at)
            .select(AccountRow::as_select())
            .load::<AccountRow>(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(OverflowProviderAccount::from)
            .collect())
    }

    fn encrypted_overflow_key(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<EncryptedOverflowKey>> {
        Ok(overflow_provider_accounts::table
            .filter(overflow_provider_accounts::workspace_id.eq(workspace_id))
            .find(id)
            .select(overflow_provider_accounts::key_ciphertext)
            .first::<Vec<u8>>(&mut *self.connection()?)
            .optional()
            .store()?
            .map(EncryptedOverflowKey::new))
    }

    fn reject_overflow_key(&self, workspace_id: Uuid, id: Uuid, at: DateTime<Utc>) -> Result<()> {
        let count = diesel::update(
            overflow_provider_accounts::table
                .filter(overflow_provider_accounts::workspace_id.eq(workspace_id))
                .find(id),
        )
        .set(overflow_provider_accounts::rejected_at.eq(at))
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }

    fn disconnect_overflow_provider(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
        at: DateTime<Utc>,
    ) -> Result<()> {
        self.connection()?
            .transaction(|connection| {
                if lock_workspace(connection, workspace_id)
                    .optional()?
                    .is_none()
                {
                    return Ok(());
                }
                retire_keys(connection, workspace_id, Some(provider), at)
            })
            .store()
    }

    fn delete_unused_overflow_keys(&self, workspace_id: Uuid) -> Result<usize> {
        erase_unused_keys(&mut *self.connection()?, workspace_id).store()
    }
}

/// Replace a provider key and record its first offer under the workspace lock.
pub(crate) fn replace_key(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    account: &OverflowProviderAccount,
    key: &EncryptedOverflowKey,
    limit: i64,
) -> QueryResult<()> {
    save_limit(connection, workspace_id, account.provider, limit)?;
    offer(connection, workspace_id, account.provider)?;
    retire_keys(
        connection,
        workspace_id,
        Some(account.provider),
        account.connected_at,
    )?;
    diesel::insert_into(overflow_provider_accounts::table)
        .values((
            overflow_provider_accounts::id.eq(account.id),
            overflow_provider_accounts::workspace_id.eq(workspace_id),
            overflow_provider_accounts::provider.eq(DbOverflowProvider(account.provider)),
            overflow_provider_accounts::account_name.eq(&account.account_name),
            overflow_provider_accounts::key_ciphertext.eq(key.as_bytes()),
            overflow_provider_accounts::connected_at.eq(account.connected_at),
        ))
        .execute(connection)?;
    Ok(())
}

/// Retire active keys while preserving ciphertext needed by live sandboxes.
pub(crate) fn retire_keys(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    provider: Option<OverflowProvider>,
    at: DateTime<Utc>,
) -> QueryResult<()> {
    diesel::sql_query(
        "UPDATE overflow_provider_accounts SET retired_at = $2
         WHERE workspace_id = $1 AND retired_at IS NULL AND ($3 IS NULL OR provider = $3)",
    )
    .bind::<sql_types::Uuid, _>(workspace_id)
    .bind::<Timestamptz, _>(at)
    .bind::<Nullable<Text>, _>(provider.map(DbOverflowProvider))
    .execute(connection)?;
    Ok(())
}

/// Erase retired keys after pending workspace runner inserts finish.
///
/// A separate lock statement gives the later DELETE a current snapshot.
pub(crate) fn erase_unused_keys(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<usize> {
    connection.transaction(|connection| {
        let workspace = workspaces::table
            .find(workspace_id)
            .select(workspaces::id)
            .for_update()
            .first::<Uuid>(connection)
            .optional()?;
        if workspace.is_none() {
            return Ok(0);
        }
        diesel::sql_query(
            "DELETE FROM overflow_provider_accounts AS account
             WHERE account.workspace_id = $1 AND account.retired_at IS NOT NULL
             AND NOT EXISTS (
                 SELECT 1 FROM runners
                 WHERE runners.workspace_id = account.workspace_id
                 AND runners.overflow_key_id = account.id AND runners.state <> 'stopped'
             ) AND NOT EXISTS (
                 SELECT 1 FROM overflow_starts
                 WHERE overflow_starts.workspace_id = account.workspace_id
                 AND overflow_starts.key_id = account.id AND overflow_starts.ends_at IS NULL
             )",
        )
        .bind::<sql_types::Uuid, _>(workspace_id)
        .execute(connection)
    })
}
