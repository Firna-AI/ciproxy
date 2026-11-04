//! Scale set queries and dedicated advisory lock connections.

use ciproxy_schema::scale_sets::scale_sets;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::scale_sets::{ScaleSet, ScaleSetSession, ScaleSetStore};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

#[derive(Queryable, Selectable)]
#[diesel(table_name = scale_sets)]
struct ScaleSetRow {
    id: Uuid,
    label_id: Uuid,
    github_id: i64,
    runner_group_id: i64,
}

impl From<ScaleSetRow> for ScaleSet {
    fn from(row: ScaleSetRow) -> Self {
        Self {
            id: row.id,
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
    fn create_scale_set(&self, organization_id: Uuid, scale_set: &ScaleSet) -> Result<()> {
        diesel::insert_into(scale_sets::table)
            .values((
                scale_sets::id.eq(scale_set.id),
                scale_sets::organization_id.eq(organization_id),
                scale_sets::label_id.eq(scale_set.label_id),
                scale_sets::github_id.eq(scale_set.github_id),
                scale_sets::runner_group_id.eq(scale_set.runner_group_id),
            ))
            .execute(&mut *self.connection()?)
            .store()?;
        Ok(())
    }

    fn scale_sets(&self, organization_id: Uuid) -> Result<Vec<ScaleSet>> {
        Ok(scale_sets::table
            .filter(scale_sets::organization_id.eq(organization_id))
            .order(scale_sets::github_id)
            .select(ScaleSetRow::as_select())
            .load::<ScaleSetRow>(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(ScaleSet::from)
            .collect())
    }

    fn try_lock_scale_set(
        &self,
        organization_id: Uuid,
        scale_set_id: Uuid,
    ) -> Result<Option<Box<dyn ScaleSetSession>>> {
        let key = scale_sets::table
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
