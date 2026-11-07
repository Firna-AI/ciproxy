//! Queries over shared saved ownership and terminal delivery proof.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Timestamptz};
use uuid::Uuid;

#[derive(QueryableByName)]
struct Access {
    #[diesel(sql_type = Bool)]
    github_access_ended: bool,
}

/// A stale take-out grants no authority over a newer installation.
/// The retained organization connects an older intent to restored access.
pub(crate) fn access_ended(
    connection: &mut PgConnection,
    workspace: Uuid,
    runner: Uuid,
) -> QueryResult<bool> {
    let access = diesel::sql_query(
        "SELECT github_access_ended FROM boat_registration_ownership
         WHERE workspace_id=$1 AND runner_id=$2",
    )
    .bind::<diesel::sql_types::Uuid, _>(workspace)
    .bind::<diesel::sql_types::Uuid, _>(runner)
    .get_result::<Access>(connection)
    .optional()?;
    Ok(access.is_none_or(|access| access.github_access_ended))
}

/// Remove only registration metadata. Preserve delivery and job history.
pub(crate) fn purge(
    connection: &mut PgConnection,
    workspace: Uuid,
    at: DateTime<Utc>,
) -> QueryResult<usize> {
    diesel::sql_query(
        "DELETE FROM boat_runner_intents i USING boat_registration_ownership o
         WHERE i.workspace_id=$1 AND o.workspace_id=i.workspace_id
         AND o.runner_id=i.runner_id AND o.terminal AND o.forget_after <= $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(workspace)
    .bind::<Timestamptz, _>(at)
    .execute(connection)
}

/// Keep the existing retention authority when organization deletion removes it.
/// Unfinished deliveries still need their exact registration identities.
pub(crate) fn retain_deadlines(
    connection: &mut PgConnection,
    workspace: Uuid,
    at: DateTime<Utc>,
) -> QueryResult<()> {
    diesel::sql_query(
        "UPDATE boat_runner_intents i SET delete_after=o.delete_after
         FROM organizations o WHERE i.workspace_id=$1
         AND o.workspace_id=i.workspace_id AND o.id=i.organization_id
         AND o.app_state='removed' AND o.removed_at IS NOT NULL
         AND o.delete_after <= $2 AND i.installation_id IS NOT NULL",
    )
    .bind::<diesel::sql_types::Uuid, _>(workspace)
    .bind::<Timestamptz, _>(at)
    .execute(connection)?;
    Ok(())
}
