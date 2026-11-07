//! Billing queries and dedicated PostgreSQL session locks.

use ciproxy_schema::billing::billing_accounts;
use ciproxy_store_interface::billing::store::{BillingRecord, BillingSession, BillingStore};
use ciproxy_store_interface::error::{Result, ResultExt};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

use super::rows;
use super::session::Session;

#[derive(QueryableByName)]
struct LockResult {
    #[diesel(sql_type = Bool)]
    acquired: bool,
}

impl BillingStore for PgStore {
    fn billing(&self, workspace_id: Uuid) -> Result<BillingRecord> {
        rows::read(&mut *self.connection()?, workspace_id)
    }

    fn billing_workspace_for_subscription(&self, subscription_id: &str) -> Result<Option<Uuid>> {
        billing_accounts::table
            .filter(billing_accounts::subscription_id.eq(subscription_id))
            .select(billing_accounts::workspace_id)
            .first::<Uuid>(&mut *self.connection()?)
            .optional()
            .store()
    }

    fn try_lock_billing(&self, workspace_id: Uuid) -> Result<Option<Box<dyn BillingSession>>> {
        let key = billing_accounts::table
            .find(workspace_id)
            .select(billing_accounts::lock_key)
            .first::<i64>(&mut *self.connection()?)
            .store()?;
        let mut connection = PgConnection::establish(&self.database_url).internal()?;
        let lock = diesel::sql_query("SELECT pg_try_advisory_lock($1) AS acquired")
            .bind::<BigInt, _>(key)
            .get_result::<LockResult>(&mut connection)
            .store()?;
        if !lock.acquired {
            return Ok(None);
        }
        Ok(Some(Box::new(Session {
            connection,
            workspace_id,
            lock_key: key,
        })))
    }

    fn billing_cancellations(&self) -> Result<Vec<Uuid>> {
        billing_accounts::table
            .filter(billing_accounts::removal_cancel_requested.eq(true))
            .select(billing_accounts::workspace_id)
            .load::<Uuid>(&mut *self.connection()?)
            .store()
    }
}
