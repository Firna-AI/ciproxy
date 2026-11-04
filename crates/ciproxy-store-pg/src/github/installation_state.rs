//! Atomic verified login and App state for one current installation registration.

use ciproxy_schema::organizations::organizations;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::github::GithubInstallationState;
use ciproxy_store_interface::organizations::Organization;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::organization_rows::OrganizationRow;

pub(super) fn save(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
    snapshot: &GithubInstallationState,
) -> Result<Organization> {
    let state = if snapshot.suspended {
        "suspended"
    } else {
        "active"
    };
    let row = diesel::update(
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
    ))
    .returning(OrganizationRow::as_returning())
    .get_result::<OrganizationRow>(&mut *store.connection()?)
    .store()?;
    row.into_record()
}
