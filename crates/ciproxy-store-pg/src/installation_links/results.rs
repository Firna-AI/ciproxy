//! Revisioned pending results commit with proof and survive failed session writes.

use ciproxy_schema::installation_links::installation_pending_results;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::installation_results::PendingInstallationResult;
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    claim_id: Uuid,
    #[diesel(sql_type = BigInt)]
    revision: i64,
}

pub(super) fn record_on(
    connection: &mut PgConnection,
    claim: Uuid,
    requester: i64,
) -> QueryResult<()> {
    diesel::sql_query(
        "INSERT INTO installation_pending_results (claim_id,requester_id) VALUES ($1,$2)
         ON CONFLICT (claim_id,requester_id) DO UPDATE SET revision=excluded.revision",
    )
    .bind::<diesel::sql_types::Uuid, _>(claim)
    .bind::<BigInt, _>(requester)
    .execute(connection)?;
    Ok(())
}

pub(super) fn read(store: &PgStore, requester: i64) -> Result<Vec<PendingInstallationResult>> {
    Ok(diesel::sql_query(
        "SELECT p.claim_id,p.revision FROM installation_pending_results p
         JOIN installation_claims c ON c.id=p.claim_id AND c.requester_id=p.requester_id
         WHERE p.requester_id=$1 ORDER BY p.claim_id",
    )
    .bind::<BigInt, _>(requester)
    .load::<Row>(&mut *store.connection()?)
    .store()?
    .into_iter()
    .map(|row| PendingInstallationResult {
        claim_id: row.claim_id,
        revision: row.revision,
    })
    .collect())
}

/// Delete only facts captured by the session's exact snapshot and requester.
pub(crate) fn consume_on(
    connection: &mut PgConnection,
    requester: i64,
    results: &[PendingInstallationResult],
) -> QueryResult<()> {
    for result in results {
        diesel::delete(
            installation_pending_results::table
                .find((result.claim_id, requester))
                .filter(installation_pending_results::revision.eq(result.revision)),
        )
        .execute(connection)?;
    }
    Ok(())
}

pub(super) fn replace_requester_on(connection: &mut PgConnection, claim: Uuid) -> QueryResult<()> {
    diesel::delete(
        installation_pending_results::table
            .filter(installation_pending_results::claim_id.eq(claim)),
    )
    .execute(connection)?;
    Ok(())
}
