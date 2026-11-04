//! Atomic first-install workspace, label, and registration insertion.

use chrono::{DateTime, Utc};
use ciproxy_schema::github::github_initial_admin;
use ciproxy_schema::labels::labels;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::settings::settings;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::workspaces::Workspace;
use diesel::prelude::*;
use diesel::query_dsl::methods;
use diesel::result::Error;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::organization_rows::repository_columns;

pub(super) fn register(
    connection: &mut PgConnection,
    workspace: Option<&Workspace>,
    organization: &Organization,
    defaults: &[Label],
    initial_admin: i64,
) -> QueryResult<()> {
    let state: &str = organization.app_state.into();
    let (mode, ids) = repository_columns(&organization.repositories);
    if let Some(workspace) = workspace {
        diesel::insert_into(workspaces::table)
            .values((
                workspaces::id.eq(workspace.id),
                workspaces::name.eq(&workspace.name),
            ))
            .execute(connection)?;
        diesel::insert_into(settings::table)
            .values(settings::workspace_id.eq(workspace.id))
            .execute(connection)?;
        diesel::insert_into(github_initial_admin::table)
            .values((
                github_initial_admin::workspace_id.eq(workspace.id),
                github_initial_admin::github_user_id.eq(initial_admin),
            ))
            .execute(connection)?;
        for label in defaults {
            let fallback: &str = label.when_nothing_fits.into();
            diesel::insert_into(labels::table)
                .values((
                    labels::id.eq(label.id),
                    labels::workspace_id.eq(workspace.id),
                    labels::name.eq(&label.name),
                    labels::vcpus.eq(label.size.vcpus),
                    labels::memory_gb.eq(label.size.memory_gb),
                    labels::disk_gb.eq(label.size.disk_gb),
                    labels::allow_machines.eq(label.allow_machines),
                    labels::allow_boat.eq(label.allow_boat),
                    labels::when_nothing_fits.eq(fallback),
                ))
                .execute(connection)?;
        }
    }
    workspaces::table
        .find(organization.workspace_id)
        .select(workspaces::id)
        .for_update()
        .first::<Uuid>(connection)?;
    let count = methods::FilterDsl::filter(
        diesel::insert_into(organizations::table)
            .values((
                organizations::id.eq(organization.id),
                organizations::workspace_id.eq(organization.workspace_id),
                organizations::github_id.eq(organization.github_id),
                organizations::login.eq(&organization.login),
                organizations::installation_id.eq(organization.installation_id),
                organizations::app_state.eq(state),
                organizations::repository_mode.eq(mode),
                organizations::allowed_repository_ids.eq(ids),
            ))
            .on_conflict(organizations::id)
            .do_update()
            .set((
                organizations::login.eq(&organization.login),
                organizations::installation_id.eq(organization.installation_id),
                organizations::app_state.eq(state),
                organizations::removed_at.eq(None::<DateTime<Utc>>),
                organizations::delete_after.eq(None::<DateTime<Utc>>),
            )),
        organizations::workspace_id.eq(organization.workspace_id),
    )
    .execute(connection)?;
    if count == 0 {
        return Err(Error::NotFound);
    }
    diesel::update(workspaces::table.find(organization.workspace_id))
        .set(workspaces::delete_after.eq(None::<DateTime<Utc>>))
        .execute(connection)?;
    QueryResult::Ok(())
}

pub(super) fn pending_admin(store: &PgStore, workspace_id: Uuid) -> Result<Option<i64>> {
    github_initial_admin::table
        .find(workspace_id)
        .select(github_initial_admin::github_user_id)
        .first(&mut *store.connection()?)
        .optional()
        .store()
}

pub(super) fn confirm_admin(store: &PgStore, workspace_id: Uuid) -> Result<()> {
    diesel::delete(github_initial_admin::table.find(workspace_id))
        .execute(&mut *store.connection()?)
        .store()?;
    Ok(())
}
