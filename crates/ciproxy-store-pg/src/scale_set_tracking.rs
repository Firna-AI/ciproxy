//! Exact retained bindings and restart discovery for completion tracking.

use chrono::{DateTime, Utc};
use ciproxy_schema::scale_sets::scale_sets;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::scale_sets::{ScaleSet, ScaleSetBinding, ScaleSetState};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

const BINDINGS: &str = concat!(
    "SELECT s.id, s.organization_id, s.label_id, s.github_id, s.runner_group_id, ",
    "l.deleted_at IS NOT NULL AS retired, s.remote_removed_at IS NOT NULL AS removed, ",
    "EXISTS (SELECT 1 FROM github_jobs j WHERE j.workspace_id = s.workspace_id ",
    "AND j.organization_id = s.organization_id AND j.scale_set_id = s.id ",
    "AND ((j.phase >= 0 AND j.phase < 2) OR j.delivered_revision < j.revision)) ",
    "AS has_tracked_jobs FROM scale_sets s JOIN labels l ON l.id = s.label_id ",
    "AND l.workspace_id = s.workspace_id WHERE s.workspace_id = $1 ",
    "AND s.organization_id = $2"
);

#[derive(QueryableByName)]
struct BindingRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    organization_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    label_id: Uuid,
    #[diesel(sql_type = BigInt)]
    github_id: i64,
    #[diesel(sql_type = BigInt)]
    runner_group_id: i64,
    #[diesel(sql_type = Bool)]
    retired: bool,
    #[diesel(sql_type = Bool)]
    removed: bool,
    #[diesel(sql_type = Bool)]
    has_tracked_jobs: bool,
}

impl From<BindingRow> for ScaleSetBinding {
    fn from(row: BindingRow) -> Self {
        Self {
            scale_set: ScaleSet {
                id: row.id,
                organization_id: row.organization_id,
                label_id: row.label_id,
                github_id: row.github_id,
                runner_group_id: row.runner_group_id,
            },
            state: if row.removed {
                ScaleSetState::Removed
            } else if row.retired {
                ScaleSetState::Draining
            } else {
                ScaleSetState::Active
            },
            has_tracked_jobs: row.has_tracked_jobs,
        }
    }
}

pub(super) fn binding(
    store: &PgStore,
    workspace: Uuid,
    organization: Uuid,
    set: Uuid,
) -> Result<Option<ScaleSetBinding>> {
    diesel::sql_query(format!("{BINDINGS} AND s.id = $3"))
        .bind::<diesel::sql_types::Uuid, _>(workspace)
        .bind::<diesel::sql_types::Uuid, _>(organization)
        .bind::<diesel::sql_types::Uuid, _>(set)
        .get_result::<BindingRow>(&mut *store.connection()?)
        .optional()
        .store()
        .map(|row| row.map(ScaleSetBinding::from))
}

pub(super) fn listeners(
    store: &PgStore,
    workspace: Uuid,
    organization: Uuid,
) -> Result<Vec<ScaleSetBinding>> {
    Ok(diesel::sql_query(format!("WITH bindings AS ({BINDINGS}) SELECT * FROM bindings WHERE NOT removed AND (NOT retired OR has_tracked_jobs) AND EXISTS (SELECT 1 FROM organization_activation a WHERE a.organization_id=bindings.organization_id AND a.admitted) ORDER BY github_id"))
        .bind::<diesel::sql_types::Uuid, _>(workspace)
        .bind::<diesel::sql_types::Uuid, _>(organization)
        .load::<BindingRow>(&mut *store.connection()?).store()?
        .into_iter().map(ScaleSetBinding::from).collect())
}

pub(super) fn complete(
    store: &PgStore,
    workspace: Uuid,
    set: &ScaleSet,
    at: DateTime<Utc>,
) -> Result<()> {
    require_record(
        diesel::update(
            scale_sets::table
                .find(set.id)
                .filter(scale_sets::workspace_id.eq(workspace))
                .filter(scale_sets::organization_id.eq(set.organization_id))
                .filter(scale_sets::github_id.eq(set.github_id)),
        )
        .set(scale_sets::remote_removed_at.eq(Some(at)))
        .execute(&mut *store.connection()?)
        .store()?,
    )
}
