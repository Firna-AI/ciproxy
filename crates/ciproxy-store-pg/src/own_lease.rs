//! Dedicated database-session ownership for workspace execution decisions.

use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::own_runners::OwnRunnerLease;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Uuid};

use crate::connection::PgStore;
use crate::error::QueryResultExt;

#[derive(QueryableByName)]
struct Key {
    #[diesel(sql_type = BigInt)]
    lock_key: i64,
}
#[derive(QueryableByName)]
struct Acquired {
    #[diesel(sql_type = Bool)]
    acquired: bool,
}

struct Lease {
    connection: PgConnection,
}

pub(crate) fn acquire(
    store: &PgStore,
    workspace_id: uuid::Uuid,
) -> Result<Option<Box<dyn OwnRunnerLease>>> {
    let mut connection = PgConnection::establish(&store.database_url).internal()?;
    let existing =
        diesel::sql_query("SELECT lock_key FROM own_machine_leases WHERE workspace_id = $1")
            .bind::<Uuid, _>(workspace_id)
            .get_result::<Key>(&mut connection)
            .optional()
            .store()?;
    let key = match existing {
        Some(key) => key,
        None => {
            diesel::sql_query("INSERT INTO own_machine_leases (workspace_id) VALUES ($1) ON CONFLICT (workspace_id) DO NOTHING")
                .bind::<Uuid, _>(workspace_id).execute(&mut connection).store()?;
            diesel::sql_query("SELECT lock_key FROM own_machine_leases WHERE workspace_id = $1")
                .bind::<Uuid, _>(workspace_id)
                .get_result::<Key>(&mut connection)
                .store()?
        }
    };
    let acquired: Acquired = diesel::sql_query("SELECT pg_try_advisory_lock($1) AS acquired")
        .bind::<BigInt, _>(key.lock_key)
        .get_result(&mut connection)
        .store()?;
    if !acquired.acquired {
        return Ok(None);
    }
    Ok(Some(Box::new(Lease { connection })))
}

impl OwnRunnerLease for Lease {
    fn check_connection(&mut self) -> Result<()> {
        diesel::sql_query("SELECT 1")
            .execute(&mut self.connection)
            .store()?;
        Ok(())
    }
}
