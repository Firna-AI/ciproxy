//! Each serving process migrates its own empty PostgreSQL database.

use std::env;

use diesel::{Connection, PgConnection, RunQueryDsl, sql_query};
use url::Url;
use uuid::Uuid;

/// An isolated empty database that the real child migrates during startup.
pub(crate) struct Database {
    /// The child-only connection URL on the configured test PostgreSQL host.
    pub(crate) url: String,
    admin: PgConnection,
    name: String,
}

impl Database {
    /// Fail when PostgreSQL is unavailable rather than omit serving assertions.
    pub(crate) fn new() -> Self {
        let raw = env::var("DATABASE_URL")
            .expect("DATABASE_URL is required for serving shutdown integration tests");
        let mut admin = PgConnection::establish(&raw).expect("connect test PostgreSQL");
        let name = format!("ciproxy_shutdown_{}", Uuid::new_v4().simple());
        sql_query(format!("CREATE DATABASE {name} TEMPLATE template0"))
            .execute(&mut admin)
            .unwrap();
        let mut url = Url::parse(&raw).unwrap();
        url.set_path(&name);
        Self {
            url: url.to_string(),
            admin,
            name,
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        sql_query(format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&mut self.admin)
            .unwrap();
    }
}
