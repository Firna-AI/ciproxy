//! Serialized proof/event joins which cannot replace a signed requester.

use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::installation_links::InstallationClaim;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamptz};
use uuid::Uuid;

use super::transaction;
use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    body: String,
}

#[derive(QueryableByName)]
struct AddEvidence {
    #[diesel(sql_type=Bool)]
    found: bool,
}

fn decode(row: Row) -> Result<InstallationClaim> {
    serde_json::from_str(&row.body).internal()
}

pub(super) fn find(store: &PgStore, id: Uuid) -> Result<Option<InstallationClaim>> {
    diesel::sql_query("SELECT row_to_json(c)::text AS body FROM installation_claims c WHERE id=$1")
        .bind::<diesel::sql_types::Uuid, _>(id)
        .get_result::<Row>(&mut *store.connection()?)
        .optional()
        .store()?
        .map(decode)
        .transpose()
}

pub(super) fn installation(store: &PgStore, id: i64) -> Result<Option<InstallationClaim>> {
    diesel::sql_query(
        "SELECT row_to_json(c)::text AS body FROM installation_claims c WHERE installation_id=$1",
    )
    .bind::<BigInt, _>(id)
    .get_result::<Row>(&mut *store.connection()?)
    .optional()
    .store()?
    .map(decode)
    .transpose()
}

pub(super) fn owned(store: &PgStore, requester: i64) -> Result<Vec<InstallationClaim>> {
    diesel::sql_query("SELECT row_to_json(c)::text AS body FROM installation_claims c WHERE requester_id=$1 ORDER BY installation_id")
        .bind::<BigInt,_>(requester).load::<Row>(&mut *store.connection()?).store()?.into_iter().map(decode).collect()
}

pub(super) fn pending_choices(store: &PgStore) -> Result<Vec<Uuid>> {
    diesel::sql_query("SELECT row_to_json(c)::text AS body FROM installation_claims c WHERE event_seen AND user_verified AND workspace_id IS NOT NULL AND organization_id IS NULL ORDER BY installation_id")
        .load::<Row>(&mut *store.connection()?).store()?.into_iter().map(|row|decode(row).map(|claim|claim.id)).collect()
}

pub(super) fn save(
    store: &PgStore,
    incoming: &InstallationClaim,
    attempt: Option<Uuid>,
) -> Result<InstallationClaim> {
    let mut connection = store.connection()?;
    transaction::run(&mut connection, |connection| {
        diesel::sql_query("SELECT pg_advisory_xact_lock(hashtextextended('ciproxy-installation-claim:' || $1::text,0))")
            .bind::<BigInt,_>(incoming.installation_id).execute(connection).store()?;
        let existing = diesel::sql_query("SELECT row_to_json(c)::text AS body FROM installation_claims c WHERE installation_id=$1 FOR UPDATE")
            .bind::<BigInt,_>(incoming.installation_id).get_result::<Row>(connection).optional().store()?.map(decode).transpose()?;
        let mut claim = incoming.clone();
        let mut linked_mode = None;
        if let Some(existing) = existing {
            if existing.github_id != incoming.github_id {
                return Err(Error::Conflict);
            }
            claim.id = existing.id;
            if existing.requester_id != incoming.requester_id {
                if existing.event_seen || !incoming.event_seen {
                    return Err(Error::Conflict);
                }
                diesel::sql_query("DELETE FROM installation_workspace_offers WHERE claim_id=$1")
                    .bind::<diesel::sql_types::Uuid, _>(claim.id)
                    .execute(connection)
                    .store()?;
                claim.user_verified = false;
                claim.direct = false;
                claim.workspace_id = None;
                claim.organization_id = None;
                claim.verified_at = None;
            } else {
                linked_mode = existing.organization_id.map(|_| existing.direct);
                claim.event_seen |= existing.event_seen;
                if incoming.verified_at.is_none() {
                    claim.user_verified |= existing.user_verified;
                }
                claim.direct |= existing.direct;
                claim.workspace_id = existing.workspace_id;
                claim.organization_id = existing.organization_id;
                claim.verified_at = claim.verified_at.or(existing.verified_at);
            }
        }
        if let Some(attempt) = attempt {
            require_record(diesel::sql_query("UPDATE installation_attempts SET installation_id=$1 WHERE id=$2 AND requester_id=$3 AND (installation_id IS NULL OR installation_id=$1)")
                .bind::<BigInt,_>(claim.installation_id).bind::<diesel::sql_types::Uuid,_>(attempt)
                .bind::<BigInt,_>(claim.requester_id).execute(connection).store()?)?;
        }
        if let Some(mode) = linked_mode {
            claim.direct = mode;
        } else {
            let add=diesel::sql_query("SELECT EXISTS(SELECT 1 FROM installation_attempts WHERE requester_id=$1 AND installation_id=$2 AND NOT direct) AS found")
                .bind::<BigInt,_>(claim.requester_id).bind::<BigInt,_>(claim.installation_id).get_result::<AddEvidence>(connection).store()?;
            if add.found {
                claim.direct = false;
            }
        }
        diesel::sql_query("INSERT INTO installation_claims (id,installation_id,github_id,login,requester_id,event_seen,user_verified,direct,workspace_id,organization_id,verified_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) ON CONFLICT (installation_id) DO UPDATE SET requester_id=excluded.requester_id,login=excluded.login,event_seen=excluded.event_seen,user_verified=excluded.user_verified,direct=excluded.direct,workspace_id=excluded.workspace_id,organization_id=excluded.organization_id,verified_at=excluded.verified_at")
            .bind::<diesel::sql_types::Uuid,_>(claim.id).bind::<BigInt,_>(claim.installation_id)
            .bind::<BigInt,_>(claim.github_id).bind::<Text,_>(&claim.login).bind::<BigInt,_>(claim.requester_id)
            .bind::<Bool,_>(claim.event_seen).bind::<Bool,_>(claim.user_verified).bind::<Bool,_>(claim.direct)
            .bind::<Nullable<diesel::sql_types::Uuid>,_>(claim.workspace_id).bind::<Nullable<diesel::sql_types::Uuid>,_>(claim.organization_id)
            .bind::<Nullable<Timestamptz>,_>(claim.verified_at).execute(connection).store()?;
        Ok(claim)
    })
}
