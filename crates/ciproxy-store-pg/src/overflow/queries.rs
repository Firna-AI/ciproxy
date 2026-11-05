//! Dedicated advisory-lock sessions and digest-only workspace authentication.

use ciproxy_schema::overflow::{overflow_locks, overflow_starts};
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::store::{OverflowSession, OverflowStore};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

use super::Session;

#[derive(QueryableByName)]
struct LockResult {
    #[diesel(sql_type = Bool)]
    acquired: bool,
}

impl OverflowStore for PgStore {
    fn try_lock_overflow(&self, workspace_id: Uuid) -> Result<Option<Box<dyn OverflowSession>>> {
        let (mut connection, key) = session_connection(self, workspace_id)?;
        let result = diesel::sql_query("SELECT pg_try_advisory_lock($1) AS acquired")
            .bind::<BigInt, _>(key)
            .get_result::<LockResult>(&mut connection)
            .store()?;
        if !result.acquired {
            return Ok(None);
        }
        Ok(Some(Box::new(Session {
            connection,
            workspace_id,
        })))
    }

    fn lock_overflow(&self, workspace_id: Uuid) -> Result<Box<dyn OverflowSession>> {
        let (mut connection, key) = session_connection(self, workspace_id)?;
        diesel::sql_query("SELECT pg_advisory_lock($1)")
            .bind::<BigInt, _>(key)
            .execute(&mut connection)
            .store()?;
        Ok(Box::new(Session {
            connection,
            workspace_id,
        }))
    }

    fn overflow_token_workspace(&self, hash: &TokenHash) -> Result<Option<Uuid>> {
        overflow_starts::table
            .filter(overflow_starts::token_hash.eq(hash.as_bytes()))
            .select(overflow_starts::workspace_id)
            .first::<Uuid>(&mut *self.connection()?)
            .optional()
            .store()
    }
}

fn session_connection(store: &PgStore, workspace_id: Uuid) -> Result<(PgConnection, i64)> {
    let key = {
        let mut connection = store.connection()?;
        diesel::insert_into(overflow_locks::table)
            .values(overflow_locks::workspace_id.eq(workspace_id))
            .on_conflict_do_nothing()
            .execute(&mut *connection)
            .store()?;
        overflow_locks::table
            .find(workspace_id)
            .select(overflow_locks::lock_key)
            .first::<i64>(&mut *connection)
            .store()?
    };
    Ok((
        PgConnection::establish(&store.database_url).internal()?,
        key,
    ))
}
