//! Shared first-removal timestamps for individual and workspace revocation.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types;
use uuid::Uuid;

/// Mark one machine or all workspace machines without changing a prior end time.
pub(crate) fn mark_removed(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Option<Uuid>,
    at: DateTime<Utc>,
) -> QueryResult<usize> {
    diesel::sql_query(
        "UPDATE machines SET removed_at = COALESCE(removed_at, $3), draining = TRUE
         WHERE workspace_id = $1 AND ($2::UUID IS NULL OR id = $2)",
    )
    .bind::<sql_types::Uuid, _>(workspace_id)
    .bind::<sql_types::Nullable<sql_types::Uuid>, _>(machine_id)
    .bind::<sql_types::Timestamptz, _>(at)
    .execute(connection)
}
