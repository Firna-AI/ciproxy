//! Each integration scenario owns a separate migrated PostgreSQL database.

use std::env;
use std::sync::Arc;

use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::prelude::*;
use url::Url;
use uuid::Uuid;

pub(super) struct Database {
    pub(super) store: Arc<dyn Store>,
    admin: PgConnection,
    name: String,
}

impl Database {
    pub(super) fn new() -> Self {
        let raw_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL is required for GitHub listener integration tests");
        let mut admin =
            PgConnection::establish(&raw_url).expect("connect isolated test PostgreSQL");
        let name = format!("ciproxy_github_{}", Uuid::new_v4().simple());
        diesel::sql_query(format!("CREATE DATABASE {name} TEMPLATE template0"))
            .execute(&mut admin)
            .unwrap();
        let mut url = Url::parse(&raw_url).unwrap();
        url.set_path(&name);
        let store = PgStoreFactory.connect(url.as_str()).unwrap();
        Self { store, admin, name }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        diesel::sql_query(format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&mut self.admin)
            .unwrap();
    }
}
