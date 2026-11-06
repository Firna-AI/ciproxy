//! Shared machine aggregate loading within the caller's read snapshot.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use ciproxy_schema::machines::{machine_disks, machine_reports, machines};
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::gateway::{MachineSnapshot, ReceivedReport};
use ciproxy_store_interface::machines::{Machine, MachineDisk};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::machine_rows::{DiskRow, MachineRow};

fn disk(row: DiskRow) -> MachineDisk {
    MachineDisk {
        path: row.path,
        total_bytes: row.total_bytes,
        free_bytes: row.free_bytes,
    }
}

/// Load summary fields and disks within the caller's transaction.
pub(crate) fn load_machines(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> Result<Vec<Machine>> {
    let rows = machines::table
        .filter(machines::workspace_id.eq(workspace_id))
        .filter(machines::removed_at.is_null())
        .order(machines::id)
        .select(MachineRow::as_select())
        .load::<MachineRow>(connection)
        .store()?;
    let disks = machine_disks::table
        .filter(machine_disks::workspace_id.eq(workspace_id))
        .order(machine_disks::path)
        .select(DiskRow::as_select())
        .load::<DiskRow>(connection)
        .store()?;
    let mut by_machine: BTreeMap<Uuid, Vec<MachineDisk>> = BTreeMap::new();
    for row in disks {
        by_machine
            .entry(row.machine_id)
            .or_default()
            .push(disk(row));
    }
    Ok(rows
        .into_iter()
        .map(|row| {
            let disks = by_machine.remove(&row.id()).unwrap_or_default();
            row.into_record(disks)
        })
        .collect())
}

/// Read one complete machine back after an authenticated contact write.
pub(crate) fn load_machine(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Uuid,
) -> Result<Machine> {
    let row = machines::table
        .filter(machines::workspace_id.eq(workspace_id))
        .filter(machines::removed_at.is_null())
        .find(machine_id)
        .select(MachineRow::as_select())
        .first::<MachineRow>(connection)
        .store()?;
    let disks = machine_disks::table
        .filter(machine_disks::workspace_id.eq(workspace_id))
        .filter(machine_disks::machine_id.eq(machine_id))
        .order(machine_disks::path)
        .select(DiskRow::as_select())
        .load::<DiskRow>(connection)
        .store()?;
    Ok(row.into_record(disks.into_iter().map(disk).collect()))
}

/// Add every active machine's full report through one workspace report query.
pub(crate) fn load_snapshots(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> Result<Vec<MachineSnapshot>> {
    let machines = load_machines(connection, workspace_id)?;
    let reports = machine_reports::table
        .filter(machine_reports::workspace_id.eq(workspace_id))
        .select((
            machine_reports::machine_id,
            machine_reports::report,
            machine_reports::received_at,
        ))
        .load::<(Uuid, serde_json::Value, DateTime<Utc>)>(connection)
        .store()?;
    let mut reports: BTreeMap<Uuid, ReceivedReport> = reports
        .into_iter()
        .map(|(machine_id, report, received_at)| {
            Ok((
                machine_id,
                ReceivedReport {
                    report: serde_json::from_value(report).internal()?,
                    received_at,
                },
            ))
        })
        .collect::<Result<_>>()?;
    machines
        .into_iter()
        .map(|machine| {
            let report = reports.remove(&machine.id);
            Ok(MachineSnapshot { machine, report })
        })
        .collect()
}
