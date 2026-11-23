//! Shared workspace routing settings queries.

use ciproxy_schema::settings::settings;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::settings::{Settings, SettingsStore};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::transaction;
use crate::overflow_provider_settings::save_limit;
use crate::workspaces::lock_workspace;

#[derive(Queryable, Selectable)]
#[diesel(table_name = settings)]
pub(crate) struct SettingsRow {
    overflow_enabled: bool,
    overflow_wait: String,
}

impl SettingsRow {
    pub(crate) fn into_record(self) -> Result<Settings> {
        Ok(Settings {
            overflow_enabled: self.overflow_enabled,
            overflow_wait: self.overflow_wait.parse().internal()?,
        })
    }
}

impl SettingsStore for PgStore {
    fn settings(&self, workspace_id: Uuid) -> Result<Settings> {
        settings::table
            .find(workspace_id)
            .select(SettingsRow::as_select())
            .first::<SettingsRow>(&mut *self.connection()?)
            .store()?
            .into_record()
    }

    fn save_settings(&self, workspace_id: Uuid, record: &Settings) -> Result<()> {
        save(&mut *self.connection()?, workspace_id, record)
    }

    fn save_settings_with_provider_limit(
        &self,
        workspace_id: Uuid,
        record: &Settings,
        provider: OverflowProvider,
        spend_limit_usd_micros: i64,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            lock_workspace(connection, workspace_id).store()?;
            save(connection, workspace_id, record)?;
            save_limit(connection, workspace_id, provider, spend_limit_usd_micros).store()
        })
    }
}

fn save(connection: &mut PgConnection, workspace_id: Uuid, record: &Settings) -> Result<()> {
    let wait: &str = record.overflow_wait.into();
    let count = diesel::update(settings::table.find(workspace_id))
        .set((
            settings::overflow_enabled.eq(record.overflow_enabled),
            settings::overflow_wait.eq(wait),
        ))
        .execute(connection)
        .store()?;
    require_record(count)
}
