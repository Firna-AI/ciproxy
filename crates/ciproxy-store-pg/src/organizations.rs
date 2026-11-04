//! Organization installation and removal bookkeeping queries.

use chrono::{DateTime, Duration, Utc};
use ciproxy_schema::boat::boat_accounts;
use ciproxy_schema::machines::machines;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::settings::settings;
use ciproxy_schema::tokens::tokens;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::organizations::{AppState, Organization, OrganizationStore};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

#[derive(Queryable, Selectable)]
#[diesel(table_name = organizations)]
struct OrganizationRow {
    id: Uuid,
    github_id: i64,
    login: String,
    installation_id: i64,
    app_state: String,
    removed_at: Option<DateTime<Utc>>,
    delete_after: Option<DateTime<Utc>>,
}

impl OrganizationRow {
    fn into_record(self) -> Result<Organization> {
        Ok(Organization {
            id: self.id,
            github_id: self.github_id,
            login: self.login,
            installation_id: self.installation_id,
            app_state: self.app_state.parse().internal()?,
            removed_at: self.removed_at,
            delete_after: self.delete_after,
        })
    }
}

impl OrganizationStore for PgStore {
    fn install_organization(&self, organization: &Organization) -> Result<()> {
        self.connection()?
            .transaction(|connection| {
                diesel::insert_into(organizations::table)
                    .values((
                        organizations::id.eq(organization.id),
                        organizations::github_id.eq(organization.github_id),
                        organizations::login.eq(&organization.login),
                        organizations::installation_id.eq(organization.installation_id),
                        organizations::app_state.eq("active"),
                    ))
                    .on_conflict(organizations::id)
                    .do_update()
                    .set((
                        organizations::github_id.eq(organization.github_id),
                        organizations::login.eq(&organization.login),
                        organizations::installation_id.eq(organization.installation_id),
                        organizations::app_state.eq("active"),
                        organizations::removed_at.eq(None::<DateTime<Utc>>),
                        organizations::delete_after.eq(None::<DateTime<Utc>>),
                    ))
                    .execute(connection)?;
                diesel::insert_into(settings::table)
                    .values(settings::organization_id.eq(organization.id))
                    .on_conflict_do_nothing()
                    .execute(connection)?;
                QueryResult::Ok(())
            })
            .store()
    }

    fn organization(&self, organization_id: Uuid) -> Result<Option<Organization>> {
        let row = organizations::table
            .find(organization_id)
            .select(OrganizationRow::as_select())
            .first(&mut *self.connection()?)
            .optional()
            .store()?;
        row.map(OrganizationRow::into_record).transpose()
    }

    fn set_app_suspended(&self, organization_id: Uuid, suspended: bool) -> Result<()> {
        let state = if suspended {
            AppState::Suspended
        } else {
            AppState::Active
        };
        let state: &str = state.into();
        let count = diesel::update(organizations::table.find(organization_id))
            .set(organizations::app_state.eq(state))
            .execute(&mut *self.connection()?)
            .store()?;
        require_record(count)
    }

    fn organization_by_github_id(&self, github_id: i64) -> Result<Option<Organization>> {
        let row = organizations::table
            .filter(organizations::github_id.eq(github_id))
            .select(OrganizationRow::as_select())
            .first(&mut *self.connection()?)
            .optional()
            .store()?;
        row.map(OrganizationRow::into_record).transpose()
    }

    fn organization_by_installation_id(
        &self,
        installation_id: i64,
    ) -> Result<Option<Organization>> {
        let row = organizations::table
            .filter(organizations::installation_id.eq(installation_id))
            .select(OrganizationRow::as_select())
            .first(&mut *self.connection()?)
            .optional()
            .store()?;
        row.map(OrganizationRow::into_record).transpose()
    }

    fn record_app_removal(&self, organization_id: Uuid, at: DateTime<Utc>) -> Result<()> {
        let count = self
            .connection()?
            .transaction(|connection| {
                let count = diesel::update(organizations::table.find(organization_id))
                    .set((
                        organizations::app_state.eq("removed"),
                        organizations::removed_at.eq(at),
                        organizations::delete_after.eq(at + Duration::days(30)),
                    ))
                    .execute(connection)?;
                diesel::update(
                    tokens::table
                        .filter(tokens::organization_id.eq(organization_id))
                        .filter(tokens::revoked_at.is_null()),
                )
                .set(tokens::revoked_at.eq(at))
                .execute(connection)?;
                diesel::update(
                    machines::table
                        .filter(machines::organization_id.eq(organization_id))
                        .filter(machines::removed_at.is_null()),
                )
                .set((machines::removed_at.eq(at), machines::draining.eq(true)))
                .execute(connection)?;
                diesel::delete(
                    boat_accounts::table.filter(boat_accounts::organization_id.eq(organization_id)),
                )
                .execute(connection)?;
                QueryResult::Ok(count)
            })
            .store()?;
        require_record(count)
    }

    fn purge_removed_organizations(&self, at: DateTime<Utc>) -> Result<usize> {
        diesel::delete(
            organizations::table
                .filter(organizations::app_state.eq("removed"))
                .filter(organizations::delete_after.le(at)),
        )
        .execute(&mut *self.connection()?)
        .store()
    }
}
