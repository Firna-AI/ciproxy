//! One locked join of signed identity and original-requester flow evidence.

use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::installation_association::SignedInstallation;
use ciproxy_store_interface::installation_links::InstallationClaim;
use diesel::prelude::*;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

use super::{attempts, claims, transaction};

pub(super) fn signed(store: &PgStore, signed: &SignedInstallation) -> Result<InstallationClaim> {
    transaction::run(&mut *store.connection()?, |connection| {
        claims::lock(connection, signed.installation_id)?;
        let existing = claims::locked(connection, signed.installation_id)?;
        let has_add = attempts::add_evidence(
            connection,
            signed.requester_id,
            Some(signed.installation_id),
            true,
        )?;
        let replace = existing
            .as_ref()
            .is_some_and(|claim| claim.requester_id != signed.requester_id && !claim.event_seen);
        let claim = signed.associate(existing, has_add)?;
        if replace {
            diesel::sql_query("DELETE FROM installation_workspace_offers WHERE claim_id=$1")
                .bind::<diesel::sql_types::Uuid, _>(claim.id)
                .execute(connection)
                .store()?;
        }
        claims::write(connection, &claim)?;
        Ok(claim)
    })
}
