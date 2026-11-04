//! Organization settings queries and typed repository mapping.

use ciproxy_schema::settings::settings;
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::settings::{RepositoryAccess, Settings, SettingsStore};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{InternalCause, QueryResultExt};

#[derive(Queryable, Selectable)]
#[diesel(table_name = settings)]
struct SettingsRow {
    overflow_enabled: bool,
    overflow_wait: String,
    repository_mode: String,
    allowed_repository_ids: Vec<i64>,
    boat_spend_limit_usd_micros: Option<i64>,
}

impl SettingsRow {
    fn into_record(self) -> Result<Settings> {
        let repositories = match self.repository_mode.as_str() {
            "all_private" => RepositoryAccess::AllPrivate,
            "selected" => RepositoryAccess::Selected(self.allowed_repository_ids),
            _ => return Err(Error::internal(InternalCause::InvalidRow)),
        };
        Ok(Settings {
            overflow_enabled: self.overflow_enabled,
            overflow_wait: self.overflow_wait.parse().internal()?,
            repositories,
            boat_spend_limit_usd_micros: self.boat_spend_limit_usd_micros,
        })
    }
}

impl SettingsStore for PgStore {
    fn settings(&self, organization_id: Uuid) -> Result<Settings> {
        settings::table
            .find(organization_id)
            .select(SettingsRow::as_select())
            .first::<SettingsRow>(&mut *self.connection()?)
            .store()?
            .into_record()
    }

    fn save_settings(&self, organization_id: Uuid, record: &Settings) -> Result<()> {
        let (mode, ids) = match &record.repositories {
            RepositoryAccess::AllPrivate => ("all_private", Vec::new()),
            RepositoryAccess::Selected(ids) => ("selected", ids.clone()),
        };
        let wait: &str = record.overflow_wait.into();
        let saved = self
            .connection()?
            .transaction(|connection| {
                let old_limit = settings::table
                    .find(organization_id)
                    .select(settings::boat_spend_limit_usd_micros)
                    .for_update()
                    .first::<Option<i64>>(connection)?;
                if old_limit.is_some() && record.boat_spend_limit_usd_micros.is_none() {
                    return QueryResult::Ok(false);
                }
                diesel::update(settings::table.find(organization_id))
                    .set((
                        settings::overflow_enabled.eq(record.overflow_enabled),
                        settings::overflow_wait.eq(wait),
                        settings::repository_mode.eq(mode),
                        settings::allowed_repository_ids.eq(ids),
                        settings::boat_spend_limit_usd_micros
                            .eq(record.boat_spend_limit_usd_micros),
                    ))
                    .execute(connection)?;
                Ok(true)
            })
            .store()?;
        if !saved {
            return Err(Error::InvalidValue);
        }
        Ok(())
    }
}
