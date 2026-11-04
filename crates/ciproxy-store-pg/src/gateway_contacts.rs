//! Session-fenced replacement of observed machine state.

use ciproxy_schema::machines::{machine_reports, machines};
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::gateway::Contact;
use ciproxy_store_interface::machines::{Machine, MachineDisk};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::machines::replace_disks;

pub(crate) fn observed_machine(id: Uuid, contact: &Contact) -> Machine {
    let report = &contact.report;
    Machine {
        id,
        hostname: report.hostname.clone(),
        processor: report.hardware.processor.clone(),
        hardware_threads: i32::try_from(report.hardware.threads).unwrap_or(i32::MAX),
        memory_bytes: i64::try_from(report.hardware.memory_bytes).unwrap_or(i64::MAX),
        kvm_available: report.hardware.kvm,
        slots: i32::try_from(report.capacity.0).unwrap_or(i32::MAX),
        warm_runners: 2,
        draining: false,
        last_contact_at: Some(contact.at),
        agent_version: report.agent_version.clone(),
        disks: report
            .hardware
            .disks
            .iter()
            .map(|disk| MachineDisk {
                path: disk.name.clone(),
                total_bytes: i64::try_from(disk.capacity_bytes).unwrap_or(i64::MAX),
                free_bytes: i64::try_from(disk.free_bytes).unwrap_or(i64::MAX),
            })
            .collect(),
        removed_at: None,
    }
}

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
    let machine = observed_machine(machine_id, contact);
    diesel::update(
        machines::table
            .filter(machines::workspace_id.eq(workspace_id))
            .find(machine_id),
    )
    .set((
        machines::hostname.eq(&machine.hostname),
        machines::processor.eq(&machine.processor),
        machines::hardware_threads.eq(machine.hardware_threads),
        machines::memory_bytes.eq(machine.memory_bytes),
        machines::kvm_available.eq(machine.kvm_available),
        machines::slots.eq(machine.slots),
        machines::last_contact_at.eq(contact.at),
        machines::agent_version.eq(&machine.agent_version),
    ))
    .execute(connection)
    .store()?;
    replace_disks(connection, workspace_id, &machine).store()?;
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
