//! Installation repository snapshot replacement and row mapping.

use ciproxy_schema::github::github_repositories;
use ciproxy_schema::organizations::organizations;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::github::Repository;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

pub(super) fn replace(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
    repositories: &[Repository],
) -> Result<()> {
    store
        .connection()?
        .transaction(|connection| {
            replace_on(connection, workspace_id, organization_id, repositories)
        })
        .store()
}

pub(super) fn replace_on(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    organization_id: Uuid,
    repositories: &[Repository],
) -> QueryResult<()> {
    organizations::table
        .filter(organizations::workspace_id.eq(workspace_id))
        .find(organization_id)
        .select(organizations::id)
        .for_update()
        .first::<Uuid>(connection)?;
    diesel::delete(
        github_repositories::table
            .filter(github_repositories::workspace_id.eq(workspace_id))
            .filter(github_repositories::organization_id.eq(organization_id)),
    )
    .execute(connection)?;
    for repository in repositories {
        diesel::insert_into(github_repositories::table)
            .values((
                github_repositories::workspace_id.eq(workspace_id),
                github_repositories::organization_id.eq(organization_id),
                github_repositories::github_id.eq(repository.id),
                github_repositories::full_name.eq(&repository.full_name),
                github_repositories::private.eq(repository.private),
                github_repositories::default_branch.eq(&repository.default_branch),
            ))
            .execute(connection)?;
    }
    QueryResult::Ok(())
}

pub(super) fn list(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
) -> Result<Vec<Repository>> {
    let rows = github_repositories::table
        .filter(github_repositories::workspace_id.eq(workspace_id))
        .filter(github_repositories::organization_id.eq(organization_id))
        .order(github_repositories::github_id)
        .select((
            github_repositories::github_id,
            github_repositories::full_name,
            github_repositories::private,
            github_repositories::default_branch,
        ))
        .load::<(i64, String, bool, String)>(&mut *store.connection()?)
        .store()?;
    Ok(rows
        .into_iter()
        .map(|(id, full_name, private, default_branch)| Repository {
            id,
            full_name,
            private,
            default_branch,
        })
        .collect())
}
