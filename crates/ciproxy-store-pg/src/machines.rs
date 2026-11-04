//! Machine registration, report replacement, and revocation queries.

use chrono::{DateTime, Utc};
use ciproxy_schema::machines::{machine_disks, machines};
use ciproxy_schema::tokens::tokens;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::machines::{Machine, MachineDisk, MachineStore};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::read_snapshot;
use crate::machine_reads::load_machines;
use crate::machine_removal::mark_removed;
use crate::machine_rows::{DiskRow, MachineRow};

pub(crate) fn replace_disks(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Uuid,
    disks: &[MachineDisk],
) -> QueryResult<()> {
    diesel::delete(
        machine_disks::table
            .filter(machine_disks::workspace_id.eq(workspace_id))
            .filter(machine_disks::machine_id.eq(machine_id)),
    )
    .execute(connection)?;
    let rows: Vec<DiskRow> = disks
        .iter()
        .map(|disk| DiskRow {
            workspace_id,
            machine_id,
            path: disk.path.clone(),
            total_bytes: disk.total_bytes,
            free_bytes: disk.free_bytes,
        })
        .collect();
    diesel::insert_into(machine_disks::table)
        .values(&rows)
        .execute(connection)?;
    Ok(())
}

impl MachineStore for PgStore {
    fn create_machine(&self, workspace_id: Uuid, machine: &Machine) -> Result<()> {
        let row = MachineRow::from_record(workspace_id, machine);
        self.connection()?
            .transaction(|connection| {
                diesel::insert_into(machines::table)
                    .values(&row)
                    .execute(connection)?;
                replace_disks(connection, workspace_id, machine.id, &machine.disks)
            })
            .store()
    }

    fn machines(&self, workspace_id: Uuid) -> Result<Vec<Machine>> {
        read_snapshot(&mut *self.connection()?, |connection| {
            load_machines(connection, workspace_id)
        })
    }

    fn update_machine(&self, workspace_id: Uuid, machine: &Machine) -> Result<()> {
        let row = MachineRow::from_record(workspace_id, machine);
        self.connection()?
            .transaction(|connection| {
                let count = diesel::update(
                    machines::table
                        .filter(machines::workspace_id.eq(workspace_id))
                        .filter(machines::removed_at.is_null())
                        .find(machine.id),
                )
                .set(&row)
                .execute(connection)?;
                if count == 0 {
                    return Err(diesel::result::Error::NotFound);
                }
                replace_disks(connection, workspace_id, machine.id, &machine.disks)
            })
            .store()
    }

    fn remove_machine(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()> {
        let count = self
            .connection()?
            .transaction(|connection| {
                remove_machine_records(connection, workspace_id, machine_id, at)
            })
            .store()?;
        require_record(count)
    }
}

pub(crate) fn remove_machine_records(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Uuid,
    at: DateTime<Utc>,
) -> QueryResult<usize> {
    let count = mark_removed(connection, workspace_id, Some(machine_id), at)?;
    diesel::update(
        tokens::table
            .filter(tokens::workspace_id.eq(workspace_id))
            .filter(tokens::machine_id.eq(machine_id))
            .filter(tokens::revoked_at.is_null()),
    )
    .set(tokens::revoked_at.eq(at))
    .execute(connection)?;
    Ok(count)
}
