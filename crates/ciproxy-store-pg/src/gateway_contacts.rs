//! Session-fenced replacement of observed machine state.

use ciproxy_schema::machines::{machine_reports, machines};
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::gateway::Contact;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::machine_observation::{MachineObservation, observed_disks};
use crate::machines::replace_disks;

pub(crate) fn lock_machine(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Uuid,
) -> Result<()> {
    machines::table
        .filter(machines::workspace_id.eq(workspace_id))
        .filter(machines::removed_at.is_null())
        .find(machine_id)
        .select(machines::id)
        .for_update()
        .first::<Uuid>(connection)
        .store()?;
    Ok(())
}

pub(crate) fn session(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Uuid,
) -> Result<Option<Uuid>> {
    machine_reports::table
        .filter(machine_reports::workspace_id.eq(workspace_id))
        .filter(machine_reports::machine_id.eq(machine_id))
        .select(machine_reports::session_id)
        .first(connection)
        .optional()
        .store()
}

pub(crate) fn write_contact(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Uuid,
    contact: &Contact,
    replace_session: bool,
) -> Result<()> {
    lock_machine(connection, workspace_id, machine_id)?;
    if !replace_session
        && session(connection, workspace_id, machine_id)? != Some(contact.session_id)
    {
        return Err(Error::Conflict);
    }
    let observed = MachineObservation::from_contact(contact);
    diesel::update(
        machines::table
            .filter(machines::workspace_id.eq(workspace_id))
            .find(machine_id),
    )
    .set(&observed)
    .execute(connection)
    .store()?;
    replace_disks(
        connection,
        workspace_id,
        machine_id,
        &observed_disks(contact),
    )
    .store()?;
    let report = serde_json::to_value(&contact.report).internal()?;
    diesel::insert_into(machine_reports::table)
        .values((
            machine_reports::workspace_id.eq(workspace_id),
            machine_reports::machine_id.eq(machine_id),
            machine_reports::session_id.eq(contact.session_id),
            machine_reports::report.eq(&report),
        ))
        .on_conflict((machine_reports::workspace_id, machine_reports::machine_id))
        .do_update()
        .set((
            machine_reports::session_id.eq(contact.session_id),
            machine_reports::report.eq(&report),
        ))
        .execute(connection)
        .store()?;
    Ok(())
}
