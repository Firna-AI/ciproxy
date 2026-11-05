//! Per-test PostgreSQL isolation for the public billing API.

use std::env;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_interface::workspaces::Workspace;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::prelude::*;
use url::Url;
use uuid::Uuid;

pub(super) struct Database {
    pub(super) store: Arc<dyn Store>,
    connection: PgConnection,
    maintenance: PgConnection,
    name: String,
}

impl Database {
    pub(super) fn reconnect_store(&mut self) {
        let raw = env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let mut url = Url::parse(&raw).unwrap();
        url.set_path(&self.name);
        self.store = PgStoreFactory.connect(url.as_str()).unwrap();
    }

    pub(super) fn set_created_at(&mut self, at: DateTime<Utc>) {
        diesel::sql_query("UPDATE billing_accounts SET created_at = $1")
            .bind::<diesel::sql_types::Timestamptz, _>(at)
            .execute(&mut self.connection)
            .unwrap();
    }

    pub(super) fn new(workspace_id: Uuid) -> Self {
        let raw = env::var("DATABASE_URL")
            .expect("DATABASE_URL is required for billing integration tests");
        let mut maintenance = PgConnection::establish(&raw).unwrap();
        let name = format!("ciproxy_billing_{}", Uuid::new_v4().simple());
        diesel::sql_query(format!("CREATE DATABASE {name} TEMPLATE template0"))
            .execute(&mut maintenance)
            .unwrap();
        let mut url = Url::parse(&raw).unwrap();
        url.set_path(&name);
        let store = PgStoreFactory.connect(url.as_str()).unwrap();
        store
            .create_workspace(&Workspace {
                id: workspace_id,
                name: "Billing fixture".into(),
                delete_after: None,
            })
            .unwrap();
        let connection = PgConnection::establish(url.as_str()).unwrap();
        Self {
            store,
            connection,
            maintenance,
            name,
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        diesel::sql_query(format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&mut self.maintenance)
            .unwrap();
    }
}
