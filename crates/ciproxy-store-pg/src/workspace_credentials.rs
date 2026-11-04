//! Atomic database mutations for ending workspace execution access.

use chrono::{DateTime, Utc};
use ciproxy_schema::machines::machines;
use ciproxy_schema::tokens::tokens;
use diesel::prelude::*;
use uuid::Uuid;

use crate::boat::{erase_unused_keys, retire_keys};

pub(crate) fn revoke_workspace_access(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    at: DateTime<Utc>,
) -> QueryResult<()> {
    diesel::update(
        tokens::table
            .filter(tokens::workspace_id.eq(workspace_id))
            .filter(tokens::purpose.eq_any(["install", "machine"]))
            .filter(tokens::revoked_at.is_null()),
    )
    .set(tokens::revoked_at.eq(at))
    .execute(connection)?;
    diesel::update(
        machines::table
            .filter(machines::workspace_id.eq(workspace_id))
            .filter(machines::removed_at.is_null()),
    )
    .set((machines::removed_at.eq(at), machines::draining.eq(true)))
    .execute(connection)?;
    retire_keys(connection, workspace_id, at)?;
    erase_unused_keys(connection, workspace_id)?;
    Ok(())
}
