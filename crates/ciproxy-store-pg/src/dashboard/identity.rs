//! Atomic identity, complete membership observations, and access request queries.

use chrono::{DateTime, Utc};
use ciproxy_schema::dashboard::{dashboard_memberships, dashboard_users};
use ciproxy_store_interface::dashboard::records::{Membership, User};
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::workspaces::Workspace;
use diesel::prelude::*;
use diesel::sql_types;
use diesel::sql_types::{BigInt, Timestamptz};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

use super::rows::UserRow;

pub(super) fn user(store: &PgStore, id: Uuid) -> Result<Option<User>> {
    Ok(dashboard_users::table
        .find(id)
        .select(UserRow::as_select())
        .first(&mut *store.connection()?)
        .optional()
        .store()?
        .map(User::from))
}

pub(super) fn by_github(store: &PgStore, github_id: i64) -> Result<Option<User>> {
    Ok(dashboard_users::table
        .filter(dashboard_users::github_id.eq(github_id))
        .select(UserRow::as_select())
        .first(&mut *store.connection()?)
        .optional()
        .store()?
        .map(User::from))
}

pub(super) fn save(store: &PgStore, user: &User, memberships: &[Membership]) -> Result<()> {
    store
        .connection()?
        .transaction(|connection| {
            save_on(connection, user, memberships)?;
            prune_on(connection)?;
            QueryResult::Ok(())
        })
        .store()
}

pub(super) fn save_on(
    connection: &mut PgConnection,
    user: &User,
    memberships: &[Membership],
) -> QueryResult<()> {
    diesel::insert_into(dashboard_users::table)
        .values((
            dashboard_users::id.eq(user.id),
            dashboard_users::github_id.eq(user.github_id),
            dashboard_users::login.eq(&user.login),
            dashboard_users::avatar_url.eq(&user.avatar_url),
            dashboard_users::credentials.eq(&user.credentials),
            dashboard_users::checked_at.eq(user.checked_at),
        ))
        .on_conflict(dashboard_users::id)
        .do_update()
        .set((
            dashboard_users::login.eq(&user.login),
            dashboard_users::avatar_url.eq(&user.avatar_url),
            dashboard_users::credentials.eq(&user.credentials),
            dashboard_users::checked_at.eq(user.checked_at),
        ))
        .execute(connection)?;
    diesel::delete(dashboard_memberships::table.filter(dashboard_memberships::user_id.eq(user.id)))
        .execute(connection)?;
    for membership in memberships {
        diesel::insert_into(dashboard_memberships::table)
            .values((
                dashboard_memberships::user_id.eq(user.id),
                dashboard_memberships::github_id.eq(membership.github_id),
                dashboard_memberships::owner.eq(membership.owner),
            ))
            .execute(connection)?;
    }
    diesel::sql_query("DELETE FROM dashboard_roles r WHERE r.github_user_id = $1 AND NOT EXISTS (SELECT 1 FROM organizations o JOIN dashboard_memberships m ON m.github_id = o.github_id WHERE o.workspace_id = r.workspace_id AND o.app_state <> 'removed' AND m.user_id = $2)")
            .bind::<BigInt, _>(user.github_id).bind::<sql_types::Uuid, _>(user.id).execute(connection)?;
    QueryResult::Ok(())
}

pub(super) fn prune_on(connection: &mut PgConnection) -> QueryResult<()> {
    diesel::sql_query("DELETE FROM dashboard_users u WHERE NOT EXISTS (SELECT 1 FROM dashboard_roles r WHERE r.github_user_id = u.github_id)").execute(connection)?;
    Ok(())
}

pub(super) fn memberships(store: &PgStore, user_id: Uuid) -> Result<Vec<Membership>> {
    Ok(dashboard_memberships::table
        .filter(dashboard_memberships::user_id.eq(user_id))
        .select((
            dashboard_memberships::github_id,
            dashboard_memberships::owner,
        ))
        .load::<(i64, bool)>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|(github_id, owner)| Membership { github_id, owner })
        .collect())
}

pub(super) fn requests_on(
    connection: &mut PgConnection,
    user_id: Uuid,
    at: DateTime<Utc>,
) -> QueryResult<()> {
    diesel::sql_query("INSERT INTO dashboard_roles (workspace_id, github_user_id, requested_at) SELECT DISTINCT o.workspace_id, u.github_id, $2 FROM dashboard_users u JOIN dashboard_memberships m ON m.user_id = u.id JOIN organizations o ON o.github_id = m.github_id AND o.app_state <> 'removed' WHERE u.id = $1 ON CONFLICT DO NOTHING")
        .bind::<sql_types::Uuid, _>(user_id).bind::<Timestamptz, _>(at).execute(connection)?;
    Ok(())
}

#[derive(QueryableByName)]
struct WorkspaceRow {
    #[diesel(sql_type = sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Timestamptz>)]
    delete_after: Option<DateTime<Utc>>,
}

pub(super) fn workspaces(store: &PgStore, user_id: Uuid) -> Result<Vec<Workspace>> {
    Ok(diesel::sql_query("SELECT DISTINCT w.id, w.name, w.delete_after FROM workspaces w JOIN organizations o ON o.workspace_id = w.id JOIN dashboard_memberships m ON m.github_id = o.github_id WHERE m.user_id = $1 AND o.app_state <> 'removed' ORDER BY w.name")
        .bind::<sql_types::Uuid, _>(user_id).load::<WorkspaceRow>(&mut *store.connection()?).store()?
        .into_iter().map(|row| Workspace {id: row.id, name: row.name, delete_after: row.delete_after }).collect())
}

pub(super) fn due(store: &PgStore, before: DateTime<Utc>) -> Result<Vec<User>> {
    Ok(dashboard_users::table
        .filter(dashboard_users::checked_at.le(before))
        .select(UserRow::as_select())
        .load::<UserRow>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(User::from)
        .collect())
}
