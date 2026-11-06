//! Membership-constrained roles and atomic owner recovery.

use chrono::{DateTime, Utc};
use ciproxy_schema::dashboard::{dashboard_roles, dashboard_users};
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::dashboard::records::{AdminClaim, Member, Role};
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use diesel::prelude::*;
use diesel::sql_types;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

use super::rows::MemberRow;

pub(super) fn members(store: &PgStore, workspace: Uuid) -> Result<Vec<Member>> {
    diesel::sql_query("SELECT r.github_user_id, u.login, r.role, r.requested_at FROM dashboard_roles r LEFT JOIN dashboard_users u ON u.github_id = r.github_user_id WHERE r.workspace_id = $1 ORDER BY r.requested_at, r.github_user_id")
        .bind::<sql_types::Uuid, _>(workspace).load::<MemberRow>(&mut *store.connection()?).store()?
        .into_iter().map(MemberRow::into_record).collect()
}

#[derive(QueryableByName)]
struct RoleRow {
    #[diesel(sql_type = sql_types::Text)]
    role: String,
}

pub(super) fn role(store: &PgStore, workspace: Uuid, github_id: i64) -> Result<Option<Role>> {
    diesel::sql_query("SELECT r.role FROM dashboard_roles r WHERE r.workspace_id = $1 AND r.github_user_id = $2 AND r.role IS NOT NULL AND EXISTS (SELECT 1 FROM dashboard_users u JOIN dashboard_memberships m ON m.user_id = u.id JOIN organizations o ON o.github_id = m.github_id WHERE u.github_id = r.github_user_id AND o.workspace_id = r.workspace_id AND o.app_state <> 'removed')")
        .bind::<sql_types::Uuid, _>(workspace).bind::<sql_types::BigInt, _>(github_id)
        .get_result::<RoleRow>(&mut *store.connection()?).optional().store()?
        .map(|row| row.role.parse().internal()).transpose()
}

pub(super) fn first_admin(
    store: &PgStore,
    workspace: Uuid,
    github_id: i64,
    at: DateTime<Utc>,
) -> Result<()> {
    store.connection()?.transaction(|connection| {
        let inserted=diesel::sql_query("INSERT INTO dashboard_initial_admin_receipts (workspace_id) VALUES ($1) ON CONFLICT DO NOTHING")
            .bind::<sql_types::Uuid,_>(workspace).execute(connection)?;
        if inserted == 0 { return QueryResult::Ok(()); }
        diesel::insert_into(dashboard_roles::table).values((dashboard_roles::workspace_id.eq(workspace),dashboard_roles::github_user_id.eq(github_id),
            dashboard_roles::role.eq("admin"),dashboard_roles::requested_at.eq(at)))
            .on_conflict((dashboard_roles::workspace_id,dashboard_roles::github_user_id)).do_update().set(dashboard_roles::role.eq("admin"))
            .execute(connection)?;
        Ok(())
    }).store()
}

pub(super) fn save(
    store: &PgStore,
    workspace: Uuid,
    github_id: i64,
    role: Option<Role>,
    at: DateTime<Utc>,
) -> Result<()> {
    let Some(role) = role else {
        return store
            .connection()?
            .transaction(|connection| {
                diesel::delete(dashboard_roles::table.find((workspace, github_id)))
                    .execute(connection)?;
                super::identity::prune_on(connection)
            })
            .store();
    };
    let member = diesel::sql_query("SELECT u.github_id FROM dashboard_users u JOIN dashboard_memberships m ON m.user_id = u.id JOIN organizations o ON o.github_id = m.github_id WHERE u.github_id = $1 AND o.workspace_id = $2 AND o.app_state <> 'removed' LIMIT 1")
        .bind::<sql_types::BigInt, _>(github_id).bind::<sql_types::Uuid, _>(workspace)
        .get_result::<GithubId>(&mut *store.connection()?).optional().store()?;
    if member.is_none() {
        return Err(Error::InvalidReference);
    }
    let value: &str = role.into();
    diesel::insert_into(dashboard_roles::table)
        .values((
            dashboard_roles::workspace_id.eq(workspace),
            dashboard_roles::github_user_id.eq(github_id),
            dashboard_roles::role.eq(value),
            dashboard_roles::requested_at.eq(at),
        ))
        .on_conflict((
            dashboard_roles::workspace_id,
            dashboard_roles::github_user_id,
        ))
        .do_update()
        .set(dashboard_roles::role.eq(value))
        .execute(&mut *store.connection()?)
        .store()?;
    Ok(())
}

#[derive(QueryableByName)]
struct GithubId {
    #[diesel(sql_type = sql_types::BigInt)]
    #[expect(dead_code, reason = "Presence establishes current membership")]
    github_id: i64,
}

#[derive(QueryableByName)]
struct Eligibility {
    #[diesel(sql_type = sql_types::Bool)]
    eligible: bool,
}

pub(super) fn claim(
    store: &PgStore,
    workspace: Uuid,
    user: Uuid,
    at: DateTime<Utc>,
    claim: AdminClaim,
) -> Result<bool> {
    store.connection()?.transaction(|connection| {
        workspaces::table.find(workspace).select(workspaces::id).for_update().first::<Uuid>(connection)?;
        let eligible = diesel::sql_query("SELECT EXISTS (SELECT 1 FROM dashboard_memberships m JOIN organizations o ON o.github_id = m.github_id WHERE m.user_id = $1 AND m.owner AND o.workspace_id = $2 AND o.app_state <> 'removed') AND (NOT EXISTS (SELECT 1 FROM organizations o WHERE o.workspace_id = $2 AND o.app_state <> 'removed' AND NOT EXISTS (SELECT 1 FROM dashboard_memberships m WHERE m.user_id = $1 AND m.github_id = o.github_id AND m.owner)) OR ($3 AND NOT EXISTS (SELECT 1 FROM dashboard_roles r JOIN dashboard_users u ON u.github_id = r.github_user_id JOIN dashboard_memberships m ON m.user_id = u.id JOIN organizations o ON o.github_id = m.github_id WHERE r.workspace_id = $2 AND r.role = 'admin' AND o.workspace_id = $2 AND o.app_state <> 'removed'))) AS eligible")
            .bind::<sql_types::Uuid, _>(user).bind::<sql_types::Uuid, _>(workspace).bind::<sql_types::Bool,_>(claim==AdminClaim::Recovery).get_result::<Eligibility>(connection)?.eligible;
        if !eligible { return QueryResult::Ok(false); }
        let github_id = dashboard_users::table.find(user).select(dashboard_users::github_id).first::<i64>(connection)?;
        diesel::insert_into(dashboard_roles::table).values((
            dashboard_roles::workspace_id.eq(workspace), dashboard_roles::github_user_id.eq(github_id),
            dashboard_roles::role.eq("admin"), dashboard_roles::requested_at.eq(at),
        )).on_conflict((dashboard_roles::workspace_id, dashboard_roles::github_user_id)).do_update()
            .set(dashboard_roles::role.eq("admin")).execute(connection)?;
        Ok(true)
    }).store()
}

pub(super) fn has_access_record(store: &PgStore, github_id: i64) -> Result<bool> {
    Ok(dashboard_roles::table
        .filter(dashboard_roles::github_user_id.eq(github_id))
        .count()
        .get_result::<i64>(&mut *store.connection()?)
        .store()?
        > 0)
}
