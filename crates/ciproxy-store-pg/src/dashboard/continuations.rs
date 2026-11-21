//! Token-free browser continuations survive person pruning until fixed expiry.

use chrono::{DateTime, Utc};
use ciproxy_schema::dashboard::dashboard_continuations;
use ciproxy_store_interface::dashboard::records::Session;
use ciproxy_store_interface::error::Result;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

pub(super) fn save_on(
    connection: &mut PgConnection,
    session: &Session,
    claim_ids: &[Uuid],
) -> QueryResult<()> {
    diesel::insert_into(dashboard_continuations::table)
        .values((
            dashboard_continuations::session_id.eq(session.id),
            dashboard_continuations::claim_ids.eq(claim_ids),
            dashboard_continuations::expires_at.eq(session.expires_at),
        ))
        .on_conflict(dashboard_continuations::session_id)
        .do_update()
        .set((
            dashboard_continuations::claim_ids.eq(claim_ids),
            dashboard_continuations::expires_at.eq(session.expires_at),
        ))
        .execute(connection)?;
    Ok(())
}

pub(super) fn read(store: &PgStore, session: Uuid, at: DateTime<Utc>) -> Result<Vec<Uuid>> {
    Ok(dashboard_continuations::table
        .find(session)
        .filter(dashboard_continuations::expires_at.gt(at))
        .select(dashboard_continuations::claim_ids)
        .first::<Vec<Uuid>>(&mut *store.connection()?)
        .optional()
        .store()?
        .unwrap_or_default())
}

pub(super) fn expire_on(connection: &mut PgConnection, at: DateTime<Utc>) -> QueryResult<()> {
    diesel::delete(
        dashboard_continuations::table.filter(dashboard_continuations::expires_at.le(at)),
    )
    .execute(connection)?;
    Ok(())
}
