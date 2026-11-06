//! Explicit numeric job and ephemeral-runner mappings remove only duplicate facts.

use ciproxy_store_interface::metering::facts::AliasedJob;
use diesel::prelude::*;
use uuid::Uuid;

const MATCHED: &str = "j.workspace_id=g.workspace_id AND j.organization_id=g.organization_id
    AND j.github_id=(g.body->>'actions_job_id')::bigint AND j.started_at IS NOT NULL
    AND r.id=j.runner_id AND r.workspace_id=j.workspace_id AND r.organization_id=j.organization_id
    AND r.github_id=(g.body->>'runner_github_id')::bigint";

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type=diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type=diesel::sql_types::Uuid)]
    organization_id: Uuid,
}

pub(super) fn load(connection: &mut PgConnection, workspace: Uuid) -> QueryResult<Vec<AliasedJob>> {
    let sql = format!(
        "SELECT DISTINCT g.id,g.organization_id FROM github_job_attempts g,jobs j,runners r WHERE g.workspace_id=$1 AND {MATCHED}"
    );
    Ok(diesel::sql_query(sql)
        .bind::<diesel::sql_types::Uuid, _>(workspace)
        .load::<Row>(connection)?
        .into_iter()
        .map(|row| AliasedJob {
            id: row.id,
            organization_id: row.organization_id,
        })
        .collect())
}

pub(super) fn exists(
    connection: &mut PgConnection,
    workspace: Uuid,
    organization: Uuid,
    id: Uuid,
) -> QueryResult<bool> {
    let sql = format!(
        "SELECT g.id,g.organization_id FROM github_job_attempts g,jobs j,runners r WHERE g.workspace_id=$1 AND g.organization_id=$2 AND g.id=$3 AND {MATCHED} LIMIT 1"
    );
    Ok(diesel::sql_query(sql)
        .bind::<diesel::sql_types::Uuid, _>(workspace)
        .bind::<diesel::sql_types::Uuid, _>(organization)
        .bind::<diesel::sql_types::Uuid, _>(id)
        .get_result::<Row>(connection)
        .optional()?
        .is_some())
}
