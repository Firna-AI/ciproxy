//! One repeatable-read view of every persisted routing input.

use ciproxy_schema::github::github_jobs;
use ciproxy_schema::labels::labels;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::settings::settings;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::read_snapshot;
use crate::labels::LabelRow;
use crate::machine_reads::load_snapshots;
use crate::organization_rows::OrganizationRow;
use crate::own_runners::load;
use crate::settings::SettingsRow;

pub(crate) fn snapshot(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> Result<RoutingSnapshot> {
    read_snapshot(connection, |connection| {
        let organizations = organizations::table
            .filter(organizations::workspace_id.eq(workspace_id))
            .order(organizations::id)
            .select(OrganizationRow::as_select())
            .load::<OrganizationRow>(connection)
            .store()?
            .into_iter()
            .map(OrganizationRow::into_record)
            .collect::<Result<Vec<_>>>()?;
        let labels = labels::table
            .filter(labels::workspace_id.eq(workspace_id))
            .filter(labels::deleted_at.is_null())
            .order(labels::name)
            .select(LabelRow::as_select())
            .load::<LabelRow>(connection)
            .store()?
            .into_iter()
            .map(LabelRow::into_record)
            .collect::<Result<Vec<_>>>()?;
        let jobs = github_jobs::table
            .filter(github_jobs::workspace_id.eq(workspace_id))
            .order((github_jobs::assigned_at, github_jobs::job_id))
            .select(github_jobs::body)
            .load::<serde_json::Value>(connection)
            .store()?
            .into_iter()
            .map(|body| serde_json::from_value(body).internal())
            .collect::<Result<Vec<_>>>()?;
        let settings = settings::table
            .find(workspace_id)
            .select(SettingsRow::as_select())
            .first::<SettingsRow>(connection)
            .store()?
            .into_record()?;
        Ok(RoutingSnapshot {
            workspace_id,
            organizations,
            labels,
            jobs,
            settings,
            machines: load_snapshots(connection, workspace_id)?,
            runners: load(connection, workspace_id)?,
        })
    })
}
