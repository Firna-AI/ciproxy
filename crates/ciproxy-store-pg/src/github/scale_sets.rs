//! Stable internal scale-set identities and retained managed-group lookup.

use ciproxy_schema::scale_sets::scale_sets;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::scale_sets::ScaleSet;
use diesel::prelude::*;
use diesel::query_dsl::methods;
use diesel::sql_types::BigInt;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

#[derive(Queryable, Selectable)]
#[diesel(table_name = scale_sets)]
struct ScaleSetRow {
    id: Uuid,
    organization_id: Uuid,
    label_id: Uuid,
    github_id: i64,
    runner_group_id: i64,
}

#[derive(QueryableByName)]
struct GroupRow {
    #[diesel(sql_type = BigInt)]
    runner_group_id: i64,
}

pub(super) fn group(store: &PgStore, github_id: i64) -> Result<Option<i64>> {
    diesel::sql_query("SELECT s.runner_group_id FROM scale_sets s JOIN organizations o ON o.id = s.organization_id WHERE o.github_id = $1 ORDER BY (o.app_state <> 'removed') DESC, o.removed_at DESC NULLS LAST, s.lock_key DESC LIMIT 1")
        .bind::<BigInt, _>(github_id).get_result::<GroupRow>(&mut *store.connection()?)
        .optional().store().map(|row| row.map(|row| row.runner_group_id))
}

pub(super) fn save(store: &PgStore, workspace_id: Uuid, scale_set: &ScaleSet) -> Result<ScaleSet> {
    save_on(&mut *store.connection()?, workspace_id, scale_set).store()
}

pub(super) fn save_on(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    scale_set: &ScaleSet,
) -> QueryResult<ScaleSet> {
    let row = methods::FilterDsl::filter(
        diesel::insert_into(scale_sets::table)
            .values((
                scale_sets::id.eq(scale_set.id),
                scale_sets::workspace_id.eq(workspace_id),
                scale_sets::organization_id.eq(scale_set.organization_id),
                scale_sets::label_id.eq(scale_set.label_id),
                scale_sets::github_id.eq(scale_set.github_id),
                scale_sets::runner_group_id.eq(scale_set.runner_group_id),
            ))
            .on_conflict((scale_sets::organization_id, scale_sets::label_id))
            .do_update()
            .set((
                scale_sets::github_id.eq(scale_set.github_id),
                scale_sets::runner_group_id.eq(scale_set.runner_group_id),
            )),
        scale_sets::workspace_id.eq(workspace_id),
    )
    .returning(ScaleSetRow::as_returning())
    .get_result::<ScaleSetRow>(connection)?;
    Ok(ScaleSet {
        id: row.id,
        organization_id: row.organization_id,
        label_id: row.label_id,
        github_id: row.github_id,
        runner_group_id: row.runner_group_id,
    })
}
