//! Server-recorded eligible offers and one immutable workspace choice.

use ciproxy_store_interface::error::Result;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool};
use uuid::Uuid;

use super::transaction;
use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

#[derive(QueryableByName)]
struct Identity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    workspace_id: Uuid,
}

pub(super) fn save(store: &PgStore, id: Uuid, requester: i64, workspaces: &[Uuid]) -> Result<()> {
    let mut connection = store.connection()?;
    transaction::run(&mut connection, |connection| {
        for workspace in workspaces {
            diesel::sql_query("INSERT INTO installation_workspace_offers (claim_id,workspace_id) SELECT id,$3 FROM installation_claims WHERE id=$1 AND requester_id=$2 AND user_verified AND event_seen ON CONFLICT DO NOTHING")
                .bind::<diesel::sql_types::Uuid,_>(id).bind::<BigInt,_>(requester)
                .bind::<diesel::sql_types::Uuid,_>(workspace).execute(connection).store()?;
        }
        Ok(())
    })
}

pub(super) fn read(store: &PgStore, id: Uuid, requester: i64) -> Result<Vec<Uuid>> {
    Ok(diesel::sql_query("SELECT o.workspace_id FROM installation_workspace_offers o JOIN installation_claims c ON c.id=o.claim_id WHERE c.id=$1 AND c.requester_id=$2 ORDER BY o.workspace_id")
        .bind::<diesel::sql_types::Uuid,_>(id).bind::<BigInt,_>(requester)
        .load::<Identity>(&mut *store.connection()?).store()?.into_iter().map(|row|row.workspace_id).collect())
}

pub(super) fn choose(store: &PgStore, id: Uuid, requester: i64, workspace: Uuid) -> Result<()> {
    require_record(diesel::sql_query("UPDATE installation_claims c SET workspace_id=$3 WHERE id=$1 AND requester_id=$2 AND user_verified AND event_seen AND (workspace_id IS NULL OR workspace_id=$3) AND EXISTS (SELECT 1 FROM installation_workspace_offers o WHERE o.claim_id=c.id AND o.workspace_id=$3)")
        .bind::<diesel::sql_types::Uuid,_>(id).bind::<BigInt,_>(requester).bind::<diesel::sql_types::Uuid,_>(workspace)
        .execute(&mut *store.connection()?).store()?)
}

pub(super) fn authorize(store: &PgStore, id: Uuid, approved: bool) -> Result<()> {
    diesel::sql_query(
        "UPDATE installation_claims SET admin_approved=$2 WHERE id=$1 AND organization_id IS NULL",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<Bool, _>(approved)
    .execute(&mut *store.connection()?)
    .store()?;
    Ok(())
}
