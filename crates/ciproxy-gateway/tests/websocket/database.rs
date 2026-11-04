//! One isolated PostgreSQL database for each gateway transport contract.

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
    maintenance: PgConnection,
    name: String,
}

impl Database {
    pub(super) fn new() -> Self {
        let raw =
            env::var("DATABASE_URL").expect("DATABASE_URL is required for gateway transport tests");
        let mut maintenance = PgConnection::establish(&raw).expect("connect PostgreSQL");
        let name = format!("ciproxy_gateway_{}", Uuid::new_v4().simple());
        diesel::sql_query(format!("CREATE DATABASE {name} TEMPLATE template0"))
            .execute(&mut maintenance)
            .unwrap();
        let mut url = Url::parse(&raw).unwrap();
        url.set_path(&name);
        let store = PgStoreFactory.connect(url.as_str()).unwrap();
        Self {
            store,
            maintenance,
            name,
        }
    }

    pub(super) fn workspace(&self, id: Uuid) {
        self.store
            .create_workspace(&Workspace {
                id,
                name: format!("Workspace {id}"),
                delete_after: None,
            })
            .unwrap();
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        diesel::sql_query(format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&mut self.maintenance)
            .unwrap();
    }
}
