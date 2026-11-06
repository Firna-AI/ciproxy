//! Recheck mutable execution authority while reserving a new own-machine runner.

use ciproxy_schema::labels::labels;
use ciproxy_schema::machines::machines;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::labels::LabelRow;

pub(crate) fn require_authority(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    intent: &OwnRunner,
) -> Result<()> {
    if intent.runner.state != RunnerState::Starting {
        return Ok(());
    }
    let RunnerPlace::Machine { machine_id, .. } = intent.runner.place else {
        return Err(Error::InvalidReference);
    };
    workspaces::table
        .find(workspace_id)
        .select(workspaces::id)
        .for_update()
        .first::<Uuid>(connection)
        .store()?;
    let allowed = machines::table
        .find(machine_id)
        .filter(machines::workspace_id.eq(workspace_id))
        .filter(machines::removed_at.is_null())
        .filter(machines::draining.eq(false))
        .select(machines::id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()
        .store()?;
    if allowed.is_none() {
        return Err(Error::InvalidReference);
    }
    let organization = organizations::table
        .find(intent.runner.organization_id)
        .filter(organizations::workspace_id.eq(workspace_id))
        .filter(organizations::app_state.eq("active"))
        .select(organizations::id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()
        .store()?;
    if organization.is_none() {
        return Err(Error::InvalidReference);
    }
    let label = labels::table
        .find(intent.runner.label_id)
        .filter(labels::workspace_id.eq(workspace_id))
        .filter(labels::deleted_at.is_null())
        .select(LabelRow::as_select())
        .for_update()
        .first::<LabelRow>(connection)
        .store()?
        .into_record()?;
    if !label.allow_machines || label.size != intent.requested_size {
        return Err(Error::InvalidValue);
    }
    Ok(())
}
