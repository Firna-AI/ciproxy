//! Machine registration, report replacement, and revocation queries.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use ciproxy_schema::machines::{machine_disks, machines};
use ciproxy_schema::tokens::tokens;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::machines::{Machine, MachineDisk, MachineStore};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = machines, treat_none_as_null = true)]
struct MachineRow {
    id: Uuid,
    organization_id: Uuid,
    hostname: String,
    processor: String,
    hardware_threads: i32,
    memory_bytes: i64,
    kvm_available: bool,
    slots: i32,
    warm_runners: i32,
    draining: bool,
    last_contact_at: Option<DateTime<Utc>>,
    agent_version: String,
    removed_at: Option<DateTime<Utc>>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = machine_disks)]
struct DiskRow {
    organization_id: Uuid,
    machine_id: Uuid,
    path: String,
    total_bytes: i64,
    free_bytes: i64,
}

impl MachineRow {
    fn from_record(organization_id: Uuid, machine: &Machine) -> Self {
        Self {
            id: machine.id,
            organization_id,
            hostname: machine.hostname.clone(),
            processor: machine.processor.clone(),
            hardware_threads: machine.hardware_threads,
            memory_bytes: machine.memory_bytes,
            kvm_available: machine.kvm_available,
            slots: machine.slots,
            warm_runners: machine.warm_runners,
            draining: machine.draining,
            last_contact_at: machine.last_contact_at,
            agent_version: machine.agent_version.clone(),
            removed_at: machine.removed_at,
        }
    }

    fn into_record(self, disks: Vec<MachineDisk>) -> Machine {
        Machine {
            id: self.id,
            hostname: self.hostname,
            processor: self.processor,
            hardware_threads: self.hardware_threads,
            memory_bytes: self.memory_bytes,
            kvm_available: self.kvm_available,
            slots: self.slots,
            warm_runners: self.warm_runners,
            draining: self.draining,
            last_contact_at: self.last_contact_at,
            agent_version: self.agent_version,
            disks,
            removed_at: self.removed_at,
        }
    }
}

fn replace_disks(
    connection: &mut PgConnection,
    organization_id: Uuid,
    machine: &Machine,
) -> QueryResult<()> {
    diesel::delete(
        machine_disks::table
            .filter(machine_disks::organization_id.eq(organization_id))
            .filter(machine_disks::machine_id.eq(machine.id)),
    )
    .execute(connection)?;
    let rows: Vec<DiskRow> = machine
        .disks
        .iter()
        .map(|disk| DiskRow {
            organization_id,
            machine_id: machine.id,
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
    fn create_machine(&self, organization_id: Uuid, machine: &Machine) -> Result<()> {
        let row = MachineRow::from_record(organization_id, machine);
        self.connection()?
            .transaction(|connection| {
                diesel::insert_into(machines::table)
                    .values(&row)
                    .execute(connection)?;
                replace_disks(connection, organization_id, machine)
            })
            .store()
    }

    fn machines(&self, organization_id: Uuid) -> Result<Vec<Machine>> {
        let mut connection = self.connection()?;
        let rows = machines::table
            .filter(machines::organization_id.eq(organization_id))
            .filter(machines::removed_at.is_null())
            .order(machines::id)
            .select(MachineRow::as_select())
            .load::<MachineRow>(&mut *connection)
            .store()?;
        let disks = machine_disks::table
            .filter(machine_disks::organization_id.eq(organization_id))
            .order(machine_disks::path)
            .select(DiskRow::as_select())
            .load::<DiskRow>(&mut *connection)
            .store()?;
        let mut by_machine: BTreeMap<Uuid, Vec<MachineDisk>> = BTreeMap::new();
        for disk in disks {
            by_machine
                .entry(disk.machine_id)
                .or_default()
                .push(MachineDisk {
                    path: disk.path,
                    total_bytes: disk.total_bytes,
                    free_bytes: disk.free_bytes,
                });
        }
        Ok(rows
            .into_iter()
            .map(|row| {
                let disks = by_machine.remove(&row.id).unwrap_or_default();
                row.into_record(disks)
            })
            .collect())
    }

    fn update_machine(&self, organization_id: Uuid, machine: &Machine) -> Result<()> {
        let row = MachineRow::from_record(organization_id, machine);
        self.connection()?
            .transaction(|connection| {
                let count = diesel::update(
                    machines::table
                        .filter(machines::organization_id.eq(organization_id))
                        .filter(machines::removed_at.is_null())
                        .find(machine.id),
                )
                .set(&row)
                .execute(connection)?;
                if count == 0 {
                    return Err(diesel::result::Error::NotFound);
                }
                replace_disks(connection, organization_id, machine)
            })
            .store()
    }

    fn remove_machine(
        &self,
        organization_id: Uuid,
        machine_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()> {
        let count = self
            .connection()?
            .transaction(|connection| {
                let count = diesel::update(
                    machines::table
                        .filter(machines::organization_id.eq(organization_id))
                        .find(machine_id),
                )
                .set((machines::removed_at.eq(at), machines::draining.eq(true)))
                .execute(connection)?;
                diesel::update(
                    tokens::table
                        .filter(tokens::organization_id.eq(organization_id))
                        .filter(tokens::machine_id.eq(machine_id))
                        .filter(tokens::revoked_at.is_null()),
                )
                .set(tokens::revoked_at.eq(at))
                .execute(connection)?;
                QueryResult::Ok(count)
            })
            .store()?;
        require_record(count)
    }
}
