//! Durable typed entry evidence, with no latest-attempt workspace selection.

use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::installation_links::InstallationAttempt;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Binary, Bool, Nullable, Uuid};

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type=Nullable<BigInt>)]
    installation_id: Option<i64>,
    #[diesel(sql_type = Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = Binary)]
    state_digest: Vec<u8>,
    #[diesel(sql_type = BigInt)]
    requester_id: i64,
    #[diesel(sql_type = Bool)]
    direct: bool,
}

pub(super) fn save(store: &PgStore, attempt: &InstallationAttempt) -> Result<()> {
    diesel::sql_query("INSERT INTO installation_attempts (id,state_digest,requester_id,direct) VALUES ($1,$2,$3,$4)")
        .bind::<Uuid,_>(attempt.id).bind::<Binary,_>(&attempt.state_digest)
        .bind::<BigInt,_>(attempt.requester_id).bind::<Bool,_>(attempt.direct)
        .execute(&mut *store.connection()?).store()?;
    Ok(())
}

pub(super) fn find(store: &PgStore, digest: &[u8]) -> Result<Option<InstallationAttempt>> {
    Ok(diesel::sql_query("SELECT id,state_digest,requester_id,direct,installation_id FROM installation_attempts WHERE state_digest=$1")
        .bind::<Binary,_>(digest).get_result::<Row>(&mut *store.connection()?).optional().store()?
        .map(|row| InstallationAttempt { installation_id:row.installation_id,id:row.id,state_digest:row.state_digest,requester_id:row.requester_id,direct:row.direct }))
}

#[derive(QueryableByName)]
struct Pending {
    #[diesel(sql_type = Bool)]
    pending: bool,
}

pub(super) fn pending(store: &PgStore, requester: i64) -> Result<bool> {
    Ok(diesel::sql_query("SELECT EXISTS (SELECT 1 FROM installation_attempts WHERE requester_id=$1 AND NOT direct AND installation_id IS NULL) AS pending")
        .bind::<BigInt,_>(requester).get_result::<Pending>(&mut *store.connection()?).store()?.pending)
}

pub(super) fn bound(store: &PgStore, requester: i64, installation: i64) -> Result<bool> {
    Ok(diesel::sql_query("SELECT EXISTS (SELECT 1 FROM installation_attempts WHERE requester_id=$1 AND installation_id=$2 AND NOT direct) AS pending")
        .bind::<BigInt,_>(requester).bind::<BigInt,_>(installation).get_result::<Pending>(&mut *store.connection()?).store()?.pending)
}

pub(super) fn by_id(store: &PgStore, id: uuid::Uuid) -> Result<Option<InstallationAttempt>> {
    Ok(diesel::sql_query(
        "SELECT id,state_digest,requester_id,direct,installation_id FROM installation_attempts WHERE id=$1",
    )
    .bind::<Uuid, _>(id)
    .get_result::<Row>(&mut *store.connection()?)
    .optional()
    .store()?
    .map(|row| InstallationAttempt {
                installation_id:row.installation_id,
        id: row.id,
        state_digest: row.state_digest,
        requester_id: row.requester_id,
        direct: row.direct,
    }))
}

pub(super) fn complete_personal(
    store: &PgStore,
    id: uuid::Uuid,
    requester: i64,
    installation: i64,
) -> Result<()> {
    require_record(diesel::sql_query("UPDATE installation_attempts SET installation_id=$3 WHERE id=$1 AND requester_id=$2 AND (installation_id IS NULL OR installation_id=$3)")
        .bind::<Uuid,_>(id).bind::<BigInt,_>(requester).bind::<BigInt,_>(installation).execute(&mut *store.connection()?).store()?)
}
