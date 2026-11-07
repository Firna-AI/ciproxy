//! App-removal mutations and retained organization cleanup.

use chrono::{DateTime, Duration, Utc};
use ciproxy_schema::billing::billing_accounts;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::workspaces::workspaces;
use diesel::dsl::max;
use diesel::prelude::*;
use uuid::Uuid;

use crate::boat_registration_ownership;
use crate::installation_links::retention;
use crate::workspace_credentials::revoke_workspace_access;

/// Preserve removal deadlines and end execution access after the last App.
pub(crate) fn mark_app_removed(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    organization_id: Uuid,
    at: DateTime<Utc>,
) -> QueryResult<()> {
    let previous_deadline = workspaces::table
        .find(workspace_id)
        .select(workspaces::delete_after)
        .for_update()
        .first::<Option<DateTime<Utc>>>(connection)?;
    let removed_at = organizations::table
        .filter(organizations::workspace_id.eq(workspace_id))
        .find(organization_id)
        .select(organizations::removed_at)
        .for_update()
        .first::<Option<DateTime<Utc>>>(connection)?
        .unwrap_or(at);
    diesel::update(
        organizations::table
            .filter(organizations::workspace_id.eq(workspace_id))
            .find(organization_id),
    )
    .set((
        organizations::app_state.eq("removed"),
        organizations::removed_at.eq(removed_at),
        organizations::delete_after.eq(removed_at + Duration::days(30)),
    ))
    .execute(connection)?;
    let remaining = organizations::table
        .filter(organizations::workspace_id.eq(workspace_id))
        .filter(organizations::app_state.ne("removed"))
        .count()
        .get_result::<i64>(connection)?;
    if remaining == 0 {
        diesel::update(billing_accounts::table.find(workspace_id))
            .set(billing_accounts::removal_cancel_requested.eq(true))
            .execute(connection)?;
        revoke_workspace_access(connection, workspace_id, at)?;
        let last_removal = organizations::table
            .filter(organizations::workspace_id.eq(workspace_id))
            .select(max(organizations::removed_at))
            .first::<Option<DateTime<Utc>>>(connection)?
            .unwrap_or(removed_at);
        let deadline = previous_deadline.unwrap_or(last_removal + Duration::days(30));
        diesel::update(workspaces::table.find(workspace_id))
            .set(workspaces::delete_after.eq(deadline))
            .execute(connection)?;
    }
    Ok(())
}

/// Serialize organization data deletion with workspace membership changes.
pub(crate) fn purge_retained_organizations(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    at: DateTime<Utc>,
) -> QueryResult<usize> {
    let workspace = workspaces::table
        .find(workspace_id)
        .select(workspaces::id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()?;
    if workspace.is_none() {
        return Ok(0);
    }
    boat_registration_ownership::retain_deadlines(connection, workspace_id, at)?;
    boat_registration_ownership::purge(connection, workspace_id, at)?;
    retention::purge(connection, workspace_id, Some(at))?;
    diesel::delete(
        organizations::table
            .filter(organizations::workspace_id.eq(workspace_id))
            .filter(organizations::app_state.eq("removed"))
            .filter(organizations::delete_after.le(at)),
    )
    .execute(connection)
}
