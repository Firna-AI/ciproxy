//! Composition-root connection and migration adapter.

use std::sync::{Arc, Mutex};

use ciproxy_schema::migrations::MIGRATIONS;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::store::{Store, StoreFactory};
use diesel::{Connection, PgConnection, RunQueryDsl, sql_query};
use diesel_migrations::MigrationHarness;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

/// Opens hosted PostgreSQL stores with embedded schema migrations.
#[derive(Default)]
pub struct PgStoreFactory;

impl StoreFactory for PgStoreFactory {
    fn connect(&self, database_url: &str) -> Result<Arc<dyn Store>> {
        let mut connection = PgConnection::establish(database_url).internal()?;
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
        Ok(Arc::new(PgStore {
            connection: Mutex::new(connection),
            database_url: database_url.to_owned(),
        }))
    }
}
