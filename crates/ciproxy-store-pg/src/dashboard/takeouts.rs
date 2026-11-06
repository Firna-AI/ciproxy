//! Atomic intentional removal distinct from owner removal and retention.

use ciproxy_schema::dashboard::dashboard_takeouts;
use ciproxy_schema::github::github_lifecycle_outbox;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::dashboard::records::Takeout;
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use ciproxy_store_interface::organizations::AppState;
use diesel::prelude::*;
use diesel::result;
use diesel::sql_types;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::organization_rows::OrganizationRow;

pub(super) fn begin(store: &PgStore, workspace: Uuid, organization: Uuid) -> Result<Takeout> {
    let mut connection = store.connection()?;
    let result = connection.transaction(|connection| {
        workspaces::table.find(workspace).select(workspaces::id).for_update().first::<Uuid>(connection)?;
        let row = organizations::table.find(organization).filter(organizations::workspace_id.eq(workspace))
            .select(OrganizationRow::as_select()).first::<OrganizationRow>(connection)?;
        let mut record = match row.into_record() {
            Ok(record) => record,
            Err(source) => return Err(result::Error::DeserializationError(Box::new(source))),
        };
        let previous = dashboard_takeouts::table.find(organization).select(dashboard_takeouts::installation_id)
            .first::<i64>(connection).optional()?;
        if previous==Some(record.installation_id) && record.app_state==AppState::Removed && record.delete_after.is_none() {
            return Ok(Some(Takeout { organization_id: organization, workspace_id: workspace, installation_id: record.installation_id }));
        }
        let remaining = organizations::table.filter(organizations::workspace_id.eq(workspace))
            .filter(organizations::app_state.ne("removed")).count().get_result::<i64>(connection)?;
        if record.app_state == AppState::Removed || remaining < 2 { return Ok(None); }
        diesel::insert_into(dashboard_takeouts::table).values((
            dashboard_takeouts::organization_id.eq(organization), dashboard_takeouts::workspace_id.eq(workspace),
            dashboard_takeouts::installation_id.eq(record.installation_id),
        )).on_conflict(dashboard_takeouts::organization_id).do_update().set((
            dashboard_takeouts::workspace_id.eq(workspace),dashboard_takeouts::installation_id.eq(record.installation_id),dashboard_takeouts::completed.eq(false),
        )).execute(connection)?;
        diesel::update(organizations::table.find(organization)).set(organizations::app_state.eq("removed")).execute(connection)?;
        record.app_state = AppState::Removed;
        let event = match serde_json::to_value(LifecycleEvent::Removed { organization: record.clone(), last_organization: false }) {
            Ok(event) => event,
            Err(source) => return Err(result::Error::SerializationError(Box::new(source))),
        };
        diesel::insert_into(github_lifecycle_outbox::table).values((
            github_lifecycle_outbox::github_id.eq(record.github_id), github_lifecycle_outbox::event.eq(event),
        )).execute(connection)?;
        diesel::sql_query("DELETE FROM dashboard_roles r WHERE r.workspace_id = $1 AND NOT EXISTS (SELECT 1 FROM dashboard_users u JOIN dashboard_memberships m ON m.user_id = u.id JOIN organizations o ON o.github_id = m.github_id WHERE u.github_id = r.github_user_id AND o.workspace_id = r.workspace_id AND o.app_state <> 'removed')")
            .bind::<sql_types::Uuid, _>(workspace).execute(connection)?;
        super::identity::prune_on(connection)?;
        Ok(Some(Takeout { organization_id: organization, workspace_id: workspace, installation_id: record.installation_id }))
    }).store()?;
    result.ok_or(Error::InvalidValue)
}

pub(super) fn pending(store: &PgStore) -> Result<Vec<Takeout>> {
    Ok(dashboard_takeouts::table
        .filter(dashboard_takeouts::completed.eq(false))
        .select((
            dashboard_takeouts::organization_id,
            dashboard_takeouts::workspace_id,
            dashboard_takeouts::installation_id,
        ))
        .load::<(Uuid, Uuid, i64)>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|(organization_id, workspace_id, installation_id)| Takeout {
            organization_id,
            workspace_id,
            installation_id,
        })
        .collect())
}

pub(super) fn complete(store: &PgStore, takeout: &Takeout) -> Result<()> {
    diesel::update(
        dashboard_takeouts::table
            .find(takeout.organization_id)
            .filter(dashboard_takeouts::workspace_id.eq(takeout.workspace_id))
            .filter(dashboard_takeouts::installation_id.eq(takeout.installation_id)),
    )
    .set(dashboard_takeouts::completed.eq(true))
    .execute(&mut *store.connection()?)
    .store()?;
    Ok(())
}
