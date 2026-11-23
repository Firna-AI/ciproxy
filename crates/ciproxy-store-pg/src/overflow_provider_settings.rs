//! Queries for durable limits and the first offer of each overflow provider.

use ciproxy_schema::overflow_provider_settings::overflow_provider_settings;
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow_provider_settings::{
    OverflowProviderSettings, OverflowProviderSettingsStore,
};
use diesel::prelude::*;
use diesel::sql_types::{self, Text};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;
use crate::label_places;
use crate::overflow_provider::DbOverflowProvider;
use crate::workspaces::lock_workspace;

impl OverflowProviderSettingsStore for PgStore {
    fn overflow_provider_settings(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
    ) -> Result<Option<OverflowProviderSettings>> {
        load(&mut *self.connection()?, workspace_id, provider).store()
    }

    fn save_overflow_provider_spend_limit(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
        spend_limit_usd_micros: Option<i64>,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            lock_workspace(connection, workspace_id).store()?;
            if let Some(limit) = spend_limit_usd_micros {
                save_limit(connection, workspace_id, provider, limit).store()?;
            } else if load(connection, workspace_id, provider)
                .store()?
                .is_some_and(|settings| settings.spend_limit_usd_micros.is_some())
            {
                return Err(Error::InvalidValue);
            }
            Ok(())
        })
    }

    fn offer_overflow_provider(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
    ) -> Result<()> {
        self.connection()?
            .transaction(|connection| {
                lock_workspace(connection, workspace_id)?;
                offer(connection, workspace_id, provider)
            })
            .store()
    }
}

/// Read the saved limit and offered mark for one provider.
pub(crate) fn load(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    provider: OverflowProvider,
) -> QueryResult<Option<OverflowProviderSettings>> {
    Ok(overflow_provider_settings::table
        .find((workspace_id, DbOverflowProvider(provider)))
        .select((
            overflow_provider_settings::spend_limit_usd_micros,
            overflow_provider_settings::offered_position,
        ))
        .first::<(Option<i64>, Option<i32>)>(connection)
        .optional()?
        .map(
            |(spend_limit_usd_micros, position)| OverflowProviderSettings {
                spend_limit_usd_micros,
                offered: position.is_some(),
            },
        ))
}

/// Read provider identities in the order in which the workspace first offered them.
pub(crate) fn offered_providers(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<Vec<OverflowProvider>> {
    Ok(overflow_provider_settings::table
        .filter(overflow_provider_settings::workspace_id.eq(workspace_id))
        .filter(overflow_provider_settings::offered_position.is_not_null())
        .order(overflow_provider_settings::offered_position)
        .select(overflow_provider_settings::provider)
        .load::<DbOverflowProvider>(connection)?
        .into_iter()
        .map(|provider| provider.0)
        .collect())
}

/// Save a positive limit while preserving the provider's offered position.
pub(crate) fn save_limit(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    provider: OverflowProvider,
    limit: i64,
) -> QueryResult<()> {
    diesel::insert_into(overflow_provider_settings::table)
        .values((
            overflow_provider_settings::workspace_id.eq(workspace_id),
            overflow_provider_settings::provider.eq(DbOverflowProvider(provider)),
            overflow_provider_settings::spend_limit_usd_micros.eq(limit),
        ))
        .on_conflict((
            overflow_provider_settings::workspace_id,
            overflow_provider_settings::provider,
        ))
        .do_update()
        .set(overflow_provider_settings::spend_limit_usd_micros.eq(limit))
        .execute(connection)?;
    Ok(())
}

/// Record the first offer and change its labels under the caller's workspace lock.
pub(crate) fn offer(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    provider: OverflowProvider,
) -> QueryResult<()> {
    let changed = diesel::sql_query(
        "INSERT INTO overflow_provider_settings (workspace_id, provider, offered_position)
         SELECT $1, $2, COALESCE(MAX(offered_position) + 1, 0)
         FROM overflow_provider_settings WHERE workspace_id = $1
         ON CONFLICT (workspace_id, provider) DO UPDATE
         SET offered_position = EXCLUDED.offered_position
         WHERE overflow_provider_settings.offered_position IS NULL",
    )
    .bind::<sql_types::Uuid, _>(workspace_id)
    .bind::<Text, _>(DbOverflowProvider(provider))
    .execute(connection)?;
    if changed != 0 {
        label_places::append_provider(connection, workspace_id, provider)?;
    }
    Ok(())
}
