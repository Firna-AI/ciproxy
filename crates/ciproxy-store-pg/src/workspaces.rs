//! Workspace creation and explicit credential revocation queries.

use chrono::{DateTime, Utc};
use ciproxy_schema::billing::billing_accounts;
use ciproxy_schema::metering::{metering_periods, metering_plan_costs};
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::settings::settings;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::workspaces::{Workspace, WorkspaceStore};
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::Timestamptz;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::workspace_credentials::revoke_workspace_access;

#[derive(Queryable, Selectable)]
#[diesel(table_name = workspaces)]
struct WorkspaceRow {
    id: Uuid,
    name: String,
    delete_after: Option<DateTime<Utc>>,
}

impl From<WorkspaceRow> for Workspace {
    fn from(row: WorkspaceRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            delete_after: row.delete_after,
        }
    }
}

/// Insert the tenant, initial Free billing/settings, and trusted metering facts.
pub(crate) fn create_workspace_on(
    connection: &mut PgConnection,
    workspace: &Workspace,
) -> QueryResult<()> {
    diesel::insert_into(workspaces::table)
        .values((
            workspaces::id.eq(workspace.id),
            workspaces::name.eq(&workspace.name),
        ))
        .execute(connection)?;
    let created_at = diesel::insert_into(billing_accounts::table)
        .values(billing_accounts::workspace_id.eq(workspace.id))
        .returning(billing_accounts::created_at)
        .get_result::<DateTime<Utc>>(connection)?;
    diesel::insert_into(settings::table)
        .values(settings::workspace_id.eq(workspace.id))
        .execute(connection)?;
    initialize_metering_on(connection, workspace.id, created_at)?;
    Ok(())
}

fn initialize_metering_on(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    created_at: DateTime<Utc>,
) -> QueryResult<()> {
    diesel::update(workspaces::table.find(workspace_id))
        .set(workspaces::metering_started_at.eq(created_at))
        .execute(connection)?;
    diesel::insert_into(metering_plan_costs::table)
        .values((
            metering_plan_costs::id.eq(workspace_id),
            metering_plan_costs::workspace_id.eq(workspace_id),
            metering_plan_costs::starts_at.eq(created_at),
            metering_plan_costs::monthly_usd.eq(Decimal::ZERO),
        ))
        .execute(connection)?;
    let ends_at = sql::<Timestamptz>("((")
        .bind::<Timestamptz, _>(created_at)
        .sql(" AT TIME ZONE 'UTC') + INTERVAL '1 month') AT TIME ZONE 'UTC'");
    diesel::insert_into(metering_periods::table)
        .values((
            metering_periods::workspace_id.eq(workspace_id),
            metering_periods::starts_at.eq(created_at),
            metering_periods::ends_at.eq(ends_at),
        ))
        .execute(connection)?;
    Ok(())
}

impl WorkspaceStore for PgStore {
    fn create_workspace(&self, workspace: &Workspace) -> Result<()> {
        self.connection()?
            .transaction(|connection| create_workspace_on(connection, workspace))
            .store()
    }

    fn workspace(&self, workspace_id: Uuid) -> Result<Option<Workspace>> {
        Ok(workspaces::table
            .find(workspace_id)
            .select(WorkspaceRow::as_select())
            .first::<WorkspaceRow>(&mut *self.connection()?)
            .optional()
            .store()?
            .map(Workspace::from))
    }

    fn revoke_workspace_execution_access(
        &self,
        workspace_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()> {
        self.connection()?
            .transaction(|connection| {
                workspaces::table
                    .find(workspace_id)
                    .select(workspaces::id)
                    .for_update()
                    .first::<Uuid>(connection)?;
                revoke_workspace_access(connection, workspace_id, at)
            })
            .store()
    }

    fn purge_expired_workspace(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<bool> {
        self.connection()?
            .transaction(|connection| {
                let deadline = workspaces::table
                    .find(workspace_id)
                    .select(workspaces::delete_after)
                    .for_update()
                    .first::<Option<DateTime<Utc>>>(connection)
                    .optional()?
                    .flatten();
                let Some(deadline) = deadline else {
                    return QueryResult::Ok(false);
                };
                if deadline > at {
                    return Ok(false);
                }
                let installed = organizations::table
                    .filter(organizations::workspace_id.eq(workspace_id))
                    .filter(organizations::app_state.ne("removed"))
                    .count()
                    .get_result::<i64>(connection)?;
                if installed != 0 {
                    return Ok(false);
                }
                let count = diesel::delete(
                    workspaces::table
                        .find(workspace_id)
                        .filter(workspaces::delete_after.le(at)),
                )
                .execute(connection)?;
                Ok(count != 0)
            })
            .store()
    }
}
