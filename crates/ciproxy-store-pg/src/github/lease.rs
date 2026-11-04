//! Waiting organization mutation leases in PostgreSQL's two-key namespace.

use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::OrganizationLease;
use diesel::prelude::*;
use diesel::sql_types::Integer;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

struct Lease {
    connection: PgConnection,
}

pub(super) fn acquire(store: &PgStore, github_id: i64) -> Result<Box<dyn OrganizationLease>> {
    let bytes = github_id.to_be_bytes();
    let first = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let second = i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let mut connection = PgConnection::establish(&store.database_url).internal()?;
    diesel::sql_query("SELECT pg_advisory_lock($1, $2)")
        .bind::<Integer, _>(first)
        .bind::<Integer, _>(second)
        .execute(&mut connection)
        .store()?;
    Ok(Box::new(Lease { connection }))
}

impl OrganizationLease for Lease {
    fn check_connection(&mut self) -> Result<()> {
        diesel::sql_query("SELECT 1")
            .execute(&mut self.connection)
            .store()?;
        Ok(())
    }
}
