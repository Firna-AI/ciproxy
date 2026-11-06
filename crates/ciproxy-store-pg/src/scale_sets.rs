//! Scale set queries and dedicated advisory lock connections.

use chrono::{DateTime, Utc};
use ciproxy_schema::labels::labels;
use ciproxy_schema::scale_sets::scale_sets;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::scale_sets::{
    ScaleSet, ScaleSetBinding, ScaleSetSession, ScaleSetStore,
};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::scale_set_tracking;

#[derive(Queryable, Selectable)]
#[diesel(table_name = scale_sets)]
pub(crate) struct ScaleSetRow {
    id: Uuid,
    organization_id: Uuid,
    label_id: Uuid,
    github_id: i64,
    runner_group_id: i64,
}

impl From<ScaleSetRow> for ScaleSet {
    fn from(row: ScaleSetRow) -> Self {
        Self {
            id: row.id,
            organization_id: row.organization_id,
            label_id: row.label_id,
            github_id: row.github_id,
            runner_group_id: row.runner_group_id,
        }
    }
}

struct Session {
    connection: PgConnection,
}

#[derive(QueryableByName)]
struct LockResult {
    #[diesel(sql_type = Bool)]
    acquired: bool,
}

impl ScaleSetSession for Session {
    fn check_connection(&mut self) -> Result<()> {
        diesel::sql_query("SELECT TRUE AS acquired")
            .get_result::<LockResult>(&mut self.connection)
            .store()?;
        Ok(())
    }
}

impl ScaleSetStore for PgStore {
    fn create_scale_set(&self, workspace_id: Uuid, scale_set: &ScaleSet) -> Result<()> {
        diesel::insert_into(scale_sets::table)
            .values((
                scale_sets::id.eq(scale_set.id),
                scale_sets::workspace_id.eq(workspace_id),
                scale_sets::organization_id.eq(scale_set.organization_id),
                scale_sets::label_id.eq(scale_set.label_id),
                scale_sets::github_id.eq(scale_set.github_id),
                scale_sets::runner_group_id.eq(scale_set.runner_group_id),
            ))
            .execute(&mut *self.connection()?)
            .store()?;
        Ok(())
    }

    fn scale_sets(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<ScaleSet>> {
        Ok(scale_sets::table
            .filter(scale_sets::workspace_id.eq(workspace_id))
            .filter(scale_sets::organization_id.eq(organization_id))
            .filter(scale_sets::remote_removed_at.is_null())
            .filter(
                scale_sets::label_id.eq_any(
                    labels::table
                        .filter(labels::deleted_at.is_null())
                        .select(labels::id),
                ),
            )
            .order(scale_sets::github_id)
            .select(ScaleSetRow::as_select())
            .load::<ScaleSetRow>(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(ScaleSet::from)
            .collect())
    }

    fn scale_set_binding(
        &self,
        workspace: Uuid,
        organization: Uuid,
        set: Uuid,
    ) -> Result<Option<ScaleSetBinding>> {
        scale_set_tracking::binding(self, workspace, organization, set)
    }

    fn listener_scale_sets(
        &self,
        workspace: Uuid,
        organization: Uuid,
    ) -> Result<Vec<ScaleSetBinding>> {
        scale_set_tracking::listeners(self, workspace, organization)
    }

    fn complete_scale_set_removal(
        &self,
        workspace: Uuid,
        set: &ScaleSet,
        at: DateTime<Utc>,
    ) -> Result<()> {
        scale_set_tracking::complete(self, workspace, set, at)
    }

    fn try_lock_scale_set(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        scale_set_id: Uuid,
    ) -> Result<Option<Box<dyn ScaleSetSession>>> {
        let key = scale_sets::table
            .filter(scale_sets::workspace_id.eq(workspace_id))
            .filter(scale_sets::organization_id.eq(organization_id))
            .find(scale_set_id)
            .select(scale_sets::lock_key)
            .first::<i64>(&mut *self.connection()?)
            .store()?;
        let mut connection = PgConnection::establish(&self.database_url).internal()?;
        let result = diesel::sql_query("SELECT pg_try_advisory_lock($1) AS acquired")
            .bind::<BigInt, _>(key)
            .get_result::<LockResult>(&mut connection)
            .store()?;
        if !result.acquired {
            return Ok(None);
        }
        Ok(Some(Box::new(Session { connection })))
    }
}
