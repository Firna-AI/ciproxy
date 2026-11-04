//! Organization installation, repository access, and removal queries.

use chrono::{DateTime, Utc};
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::organizations::{
    AppState, Organization, OrganizationStore, RepositoryAccess,
};
use diesel::prelude::*;
use diesel::query_dsl::methods;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::organization_lifecycle::{mark_app_removed, purge_retained_organizations};
use crate::organization_rows::{OrganizationRow, repository_columns};

impl OrganizationStore for PgStore {
    fn install_organization(&self, record: &Organization) -> Result<()> {
        let (mode, ids) = repository_columns(&record.repositories);
        let count = self
            .connection()?
            .transaction(|connection| {
                workspaces::table
                    .find(record.workspace_id)
                    .select(workspaces::id)
                    .for_update()
                    .first::<Uuid>(connection)?;
                let count = methods::FilterDsl::filter(
                    diesel::insert_into(organizations::table)
                        .values((
                            organizations::id.eq(record.id),
                            organizations::workspace_id.eq(record.workspace_id),
                            organizations::github_id.eq(record.github_id),
                            organizations::login.eq(&record.login),
                            organizations::installation_id.eq(record.installation_id),
                            organizations::app_state.eq("active"),
                            organizations::repository_mode.eq(mode),
                            organizations::allowed_repository_ids.eq(ids),
                        ))
                        .on_conflict(organizations::id)
                        .do_update()
                        .set((
                            organizations::github_id.eq(record.github_id),
                            organizations::login.eq(&record.login),
                            organizations::installation_id.eq(record.installation_id),
                            organizations::app_state.eq("active"),
                            organizations::removed_at.eq(None::<DateTime<Utc>>),
                            organizations::delete_after.eq(None::<DateTime<Utc>>),
                        )),
                    organizations::workspace_id.eq(record.workspace_id),
                )
                .execute(connection)?;
                if count != 0 {
                    diesel::update(workspaces::table.find(record.workspace_id))
                        .set(workspaces::delete_after.eq(None::<DateTime<Utc>>))
                        .execute(connection)?;
                }
                QueryResult::Ok(count)
            })
            .store()?;
        require_record(count)
    }

    fn organization(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Option<Organization>> {
        let row = organizations::table
            .filter(organizations::workspace_id.eq(workspace_id))
            .find(organization_id)
            .select(OrganizationRow::as_select())
            .first(&mut *self.connection()?)
            .optional()
            .store()?;
        row.map(OrganizationRow::into_record).transpose()
    }

    fn organizations(&self, workspace_id: Uuid) -> Result<Vec<Organization>> {
        organizations::table
            .filter(organizations::workspace_id.eq(workspace_id))
            .order(organizations::login)
            .select(OrganizationRow::as_select())
            .load(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(OrganizationRow::into_record)
            .collect()
    }

    fn organizations_by_github_id(&self, github_id: i64) -> Result<Vec<Organization>> {
        organizations::table
            .filter(organizations::github_id.eq(github_id))
            .order(organizations::id)
            .select(OrganizationRow::as_select())
            .load(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(OrganizationRow::into_record)
            .collect()
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

    fn set_app_suspended(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        suspended: bool,
    ) -> Result<()> {
        let state = if suspended {
            AppState::Suspended
        } else {
            AppState::Active
        };
        let state: &str = state.into();
        let count = diesel::update(
            organizations::table
                .filter(organizations::workspace_id.eq(workspace_id))
                .filter(organizations::app_state.ne("removed"))
                .find(organization_id),
        )
        .set(organizations::app_state.eq(state))
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }

    fn save_repository_access(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        repositories: &RepositoryAccess,
    ) -> Result<()> {
        let (mode, ids) = repository_columns(repositories);
        let count = diesel::update(
            organizations::table
                .filter(organizations::workspace_id.eq(workspace_id))
                .find(organization_id),
        )
        .set((
            organizations::repository_mode.eq(mode),
            organizations::allowed_repository_ids.eq(ids),
        ))
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }

    fn record_app_removal(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()> {
        self.connection()?
            .transaction(|connection| {
                mark_app_removed(connection, workspace_id, organization_id, at)
            })
            .store()
    }

    fn purge_removed_organizations(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<usize> {
        self.connection()?
            .transaction(|connection| purge_retained_organizations(connection, workspace_id, at))
            .store()
    }
}
