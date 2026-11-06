//! Purge linked installation facts with their organization's normal retention.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Timestamptz};
use uuid::Uuid;

const REGISTRATIONS: &str = "SELECT id FROM organizations WHERE workspace_id=$1 AND ($2::timestamptz IS NULL OR (app_state='removed' AND delete_after<=$2))";

/// Unlinked claims survive loss of a proposed destination and remain fail-closed.
pub(crate) fn purge(
    connection: &mut PgConnection,
    workspace: Uuid,
    before: Option<DateTime<Utc>>,
) -> QueryResult<()> {
    diesel::sql_query(format!("DELETE FROM installation_attempts WHERE installation_id IN (SELECT installation_id FROM installation_claims WHERE organization_id IN ({REGISTRATIONS}))"))
        .bind::<diesel::sql_types::Uuid,_>(workspace).bind::<Nullable<Timestamptz>,_>(before).execute(connection)?;
    diesel::sql_query(format!(
        "DELETE FROM installation_claims WHERE organization_id IN ({REGISTRATIONS})"
    ))
    .bind::<diesel::sql_types::Uuid, _>(workspace)
    .bind::<Nullable<Timestamptz>, _>(before)
    .execute(connection)?;
    Ok(())
}
