//! Isolated PostgreSQL fixture for the independent billing lock target.

use std::env;
use std::sync::Arc;

use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_interface::workspaces::Workspace;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::prelude::*;
use url::Url;
use uuid::Uuid;

pub(super) struct Database {
    pub(super) store: Arc<dyn Store>,
    pub(super) url: String,
    maintenance: PgConnection,
    name: String,
}

impl Database {
    pub(super) fn new() -> Self {
        let raw = env::var("DATABASE_URL")
            .expect("DATABASE_URL is required for billing lock integration tests");
        let mut maintenance = PgConnection::establish(&raw).unwrap();
        let name = format!("ciproxy_billing_lock_{}", Uuid::new_v4().simple());
        diesel::sql_query(format!("CREATE DATABASE {name} TEMPLATE template0"))
            .execute(&mut maintenance)
            .unwrap();
        let mut url = Url::parse(&raw).unwrap();
        url.set_path(&name);
        let url = url.to_string();
        let store = PgStoreFactory.connect(&url).unwrap();
        Self {
            store,
            url,
            maintenance,
            name,
        }
    }

    pub(super) fn workspace(&self) -> Workspace {
        let workspace = Workspace {
            id: Uuid::new_v4(),
            name: "Billing lock fixture".into(),
            delete_after: None,
        };
        self.store.create_workspace(&workspace).unwrap();
        workspace
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        diesel::sql_query(format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&mut self.maintenance)
            .unwrap();
    }
}
