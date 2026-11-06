//! Workspace intent and history-preserving label removal queries.

use crate::scale_sets::ScaleSetRow;
use chrono::{DateTime, Utc};
use ciproxy_schema::dashboard::dashboard_install_intents;
use ciproxy_schema::labels::labels;
use ciproxy_schema::scale_sets::scale_sets;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::scale_sets::ScaleSet;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::labels::LabelRow;

pub(super) fn save_intent(
    store: &PgStore,
    requester: i64,
    organization: i64,
    workspace: Uuid,
) -> Result<()> {
    diesel::insert_into(dashboard_install_intents::table)
        .values((
            dashboard_install_intents::requester_id.eq(requester),
            dashboard_install_intents::github_organization_id.eq(organization),
            dashboard_install_intents::workspace_id.eq(workspace),
        ))
        .on_conflict((
            dashboard_install_intents::requester_id,
            dashboard_install_intents::github_organization_id,
        ))
        .do_update()
        .set(dashboard_install_intents::workspace_id.eq(workspace))
        .execute(&mut *store.connection()?)
        .store()?;
    Ok(())
}

pub(super) fn intent(store: &PgStore, requester: i64, organization: i64) -> Result<Option<Uuid>> {
    dashboard_install_intents::table
        .find((requester, organization))
        .select(dashboard_install_intents::workspace_id)
        .first(&mut *store.connection()?)
        .optional()
        .store()
}

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
    let name = labels::table
        .find(label)
        .filter(labels::workspace_id.eq(workspace))
        .select(labels::name)
        .first::<String>(&mut *store.connection()?)
        .store()?;
    if name == "ciproxy" {
        return Err(Error::InvalidValue);
    }
    require_record(
        diesel::update(
            labels::table
                .find(label)
                .filter(labels::workspace_id.eq(workspace)),
        )
        .set((
            labels::deleted_at.eq(at),
            labels::allow_machines.eq(false),
            labels::allow_boat.eq(false),
        ))
        .execute(&mut *store.connection()?)
        .store()?,
    )
}

pub(super) fn retired(store: &PgStore, workspace: Uuid) -> Result<Vec<Label>> {
    labels::table
        .filter(labels::workspace_id.eq(workspace))
        .filter(labels::deleted_at.is_not_null())
        .select(LabelRow::as_select())
        .load(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(LabelRow::into_record)
        .collect()
}

pub(super) fn sets(store: &PgStore, workspace: Uuid, organization: Uuid) -> Result<Vec<ScaleSet>> {
    Ok(scale_sets::table
        .filter(scale_sets::workspace_id.eq(workspace))
        .filter(scale_sets::organization_id.eq(organization))
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
        .select(labels::workspace_id)
        .distinct()
        .load(&mut *store.connection()?)
        .store()
}
