//! Atomic database mutations for ending workspace execution access.

use chrono::{DateTime, Utc};
use ciproxy_schema::overflow::overflow_connection_grants;
use ciproxy_schema::tokens::tokens;
use diesel::prelude::*;
use uuid::Uuid;

use crate::boat::{erase_unused_keys, retire_keys};
use crate::machine_removal::mark_removed;

pub(crate) fn revoke_workspace_access(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    at: DateTime<Utc>,
) -> QueryResult<()> {
    diesel::delete(
        overflow_connection_grants::table
            .filter(overflow_connection_grants::workspace_id.eq(workspace_id)),
    )
    .execute(connection)?;
    diesel::update(
        tokens::table
            .filter(tokens::workspace_id.eq(workspace_id))
            .filter(tokens::purpose.eq_any(["install", "machine"]))
            .filter(tokens::revoked_at.is_null()),
    )
    .set(tokens::revoked_at.eq(at))
    .execute(connection)?;
    mark_removed(connection, workspace_id, None, at)?;
    retire_keys(connection, workspace_id, at)?;
    erase_unused_keys(connection, workspace_id)?;
    Ok(())
}
