//! Atomic first-install workspace, label, and registration insertion.

use chrono::{DateTime, Utc};
use ciproxy_schema::github::github_initial_admin;
use ciproxy_schema::organizations::organizations;
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
use crate::labels;
use crate::organization_rows::repository_columns;
use crate::workspaces::{create_workspace_on, lock_workspace};

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
        create_workspace_on(connection, workspace)?;
        diesel::insert_into(github_initial_admin::table)
            .values((
                github_initial_admin::workspace_id.eq(workspace.id),
                github_initial_admin::github_user_id.eq(initial_admin),
            ))
            .execute(connection)?;
        for label in defaults {
            labels::insert(connection, workspace.id, label, None)?;
        }
    }
    lock_workspace(connection, organization.workspace_id)?;
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
