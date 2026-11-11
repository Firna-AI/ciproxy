//! Scoped registration and complete repository observation provenance.

use ciproxy_schema::dashboard::dashboard_users;
use ciproxy_schema::organizations::organizations;
use ciproxy_store_interface::dashboard::facts::{MemberProfile, OrganizationFacts};
use ciproxy_store_interface::error::Result;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

pub(super) fn organization(
    store: &PgStore,
    workspace: Uuid,
    organization: Uuid,
) -> Result<OrganizationFacts> {
    let (joined_at, repositories_observed_at) = organizations::table
        .find(organization)
        .filter(organizations::workspace_id.eq(workspace))
        .select((
            organizations::joined_at,
            organizations::repositories_observed_at,
        ))
        .first(&mut *store.connection()?)
        .store()?;
    Ok(OrganizationFacts {
        joined_at,
        repositories_observed_at,
    })
}

pub(super) fn profile(store: &PgStore, github_user_id: i64) -> Result<Option<MemberProfile>> {
    let row = dashboard_users::table
        .filter(dashboard_users::github_id.eq(github_user_id))
        .select((
            dashboard_users::login,
            dashboard_users::name,
            dashboard_users::avatar_url,
        ))
        .first::<(String, Option<String>, String)>(&mut *store.connection()?)
        .optional()
        .store()?;
    Ok(row.map(|(login, name, avatar_url)| MemberProfile {
        login,
        name,
        avatar_url,
    }))
}
