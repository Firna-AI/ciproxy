//! Scoped verified identity, App state, and optional access writes in one transaction.

use ciproxy_schema::organizations::organizations;
use ciproxy_store_interface::github_lifecycle::GithubLifecycleUpdate;
use ciproxy_store_interface::organizations::Organization;
use diesel::prelude::*;
use uuid::Uuid;

use crate::organization_rows::repository_columns;

use super::rows;

pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    organization_id: Uuid,
    snapshot: &GithubLifecycleUpdate,
) -> QueryResult<Organization> {
    let previous = rows::load(connection, workspace_id, organization_id)?;
    let repositories = snapshot
        .repositories
        .as_ref()
        .unwrap_or(&previous.repositories);
    let (mode, ids) = repository_columns(repositories);
    let state: &str = rows::state(&snapshot.target).into();
    let count = diesel::update(
        organizations::table
            .find(organization_id)
            .filter(organizations::workspace_id.eq(workspace_id))
            .filter(organizations::github_id.eq(snapshot.github_id))
            .filter(organizations::installation_id.eq(snapshot.installation_id))
            .filter(organizations::app_state.ne("removed")),
    )
    .set((
        organizations::login.eq(&snapshot.login),
        organizations::app_state.eq(state),
        organizations::repository_mode.eq(mode),
        organizations::allowed_repository_ids.eq(ids),
    ))
    .execute(connection)?;
    if count == 0 {
        return Err(diesel::result::Error::NotFound);
    }
    rows::load(connection, workspace_id, organization_id)
}
