//! Workspace intent and history-preserving label removal queries.

use crate::scale_sets::ScaleSetRow;
use chrono::{DateTime, Utc};
use ciproxy_schema::labels::labels;
use ciproxy_schema::scale_sets::scale_sets;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::scale_sets::ScaleSet;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::transaction;
use crate::label_places;
use crate::workspaces::lock_workspace;

pub(super) fn rename(store: &PgStore, workspace: Uuid, name: &str) -> Result<()> {
    require_record(
        diesel::update(workspaces::table.find(workspace))
            .set(workspaces::name.eq(name))
            .execute(&mut *store.connection()?)
            .store()?,
    )
}

pub(super) fn retire(
    store: &PgStore,
    workspace: Uuid,
    label: Uuid,
    at: DateTime<Utc>,
) -> Result<()> {
    transaction(&mut *store.connection()?, |connection| {
        lock_workspace(connection, workspace).store()?;
        let owned_label = labels::table
            .find(label)
            .filter(labels::workspace_id.eq(workspace));
        let name = owned_label
            .select(labels::name)
            .first::<String>(connection)
            .store()?;
        if name == "ciproxy" {
            return Err(Error::InvalidValue);
        }
        let count = diesel::update(owned_label)
            .set(labels::deleted_at.eq(at))
            .execute(connection)
            .store()?;
        require_record(count)?;
        label_places::clear(connection, label).store()
    })
}

pub(super) fn sets(store: &PgStore, workspace: Uuid, organization: Uuid) -> Result<Vec<ScaleSet>> {
    Ok(scale_sets::table
        .filter(scale_sets::workspace_id.eq(workspace))
        .filter(scale_sets::organization_id.eq(organization))
        .filter(scale_sets::remote_removed_at.is_null())
        .select(ScaleSetRow::as_select())
        .load::<ScaleSetRow>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(ScaleSet::from)
        .collect())
}

pub(super) fn retired_workspaces(store: &PgStore) -> Result<Vec<Uuid>> {
    labels::table
        .filter(labels::deleted_at.is_not_null())
        .filter(
            labels::id.eq_any(
                scale_sets::table
                    .filter(scale_sets::remote_removed_at.is_null())
                    .select(scale_sets::label_id),
            ),
        )
        .select(labels::workspace_id)
        .distinct()
        .load(&mut *store.connection()?)
        .store()
}
