//! Composition-root connection and migration adapter.

use std::sync::{Arc, Mutex};

use ciproxy_schema::migrations::MIGRATIONS;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::job_usage::JobUsageStore;
use ciproxy_store_interface::store::{Store, StoreFactory};
use diesel::{RunQueryDsl, sql_query};
use diesel_migrations::MigrationHarness;

use crate::connection::PgStore;
use crate::connector::{BoundedConnector, ConnectionUse, Connector};
use crate::error::QueryResultExt;

/// Opens hosted PostgreSQL stores with embedded schema migrations.
#[derive(Default)]
pub struct PgStoreFactory;

impl StoreFactory for PgStoreFactory {
    fn connect(&self, database_url: &str) -> Result<Arc<dyn Store>> {
        Ok(self.connect_with_job_usage(database_url)?)
    }
}

impl PgStoreFactory {
    /// Connect once for both the general store and the separate job usage trait.
    pub fn connect_with_job_usage(
        &self,
        database_url: &str,
    ) -> Result<Arc<impl Store + JobUsageStore + use<>>> {
        let connector: Box<dyn Connector> = Box::new(BoundedConnector::new(database_url));
        {
            let mut connection = connector.connect(ConnectionUse::Migrations)?;
            sql_query("SELECT pg_advisory_lock(-1128878674)")
                .execute(&mut connection)
                .store()?;
            let migrations = connection
                .run_pending_migrations(MIGRATIONS)
                .map(|_| ())
                .internal();
            sql_query("SELECT pg_advisory_unlock(-1128878674)")
                .execute(&mut connection)
                .store()?;
            migrations?;
        }
        let connection = connector.connect(ConnectionUse::Queries)?;
        Ok(Arc::new(PgStore {
            connection: Mutex::new(connection),
            connector,
        }))
    }
}
