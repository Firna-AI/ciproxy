//! Private row mappings for complete machine state and disks.

use chrono::{DateTime, Utc};
use ciproxy_schema::machines::{machine_disks, machines};
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::machines::{Machine, MachineDisk};
use diesel::prelude::*;
use uuid::Uuid;

use crate::platform_text::{
    parse_platform, parse_virtualization, platform_text, virtualization_text,
};

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = machines, treat_none_as_null = true)]
pub(crate) struct MachineRow {
    id: Uuid,
    workspace_id: Uuid,
    hostname: String,
    processor: String,
    hardware_threads: i32,
    memory_bytes: i64,
    #[diesel(skip_update)]
    platform: String,
    virtualization: String,
    slots: i32,
    warm_runners: i32,
    draining: bool,
    last_contact_at: Option<DateTime<Utc>>,
    agent_version: String,
    removed_at: Option<DateTime<Utc>>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = machine_disks)]
pub(crate) struct DiskRow {
    pub(crate) workspace_id: Uuid,
    pub(crate) machine_id: Uuid,
    pub(crate) path: String,
    pub(crate) total_bytes: i64,
    pub(crate) free_bytes: i64,
}

impl MachineRow {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn from_record(workspace_id: Uuid, machine: &Machine) -> Self {
        Self {
            id: machine.id,
            workspace_id,
            hostname: machine.hostname.clone(),
            processor: machine.processor.clone(),
            hardware_threads: machine.hardware_threads,
            memory_bytes: machine.memory_bytes,
            platform: platform_text(machine.platform).to_owned(),
            virtualization: virtualization_text(machine.virtualization).to_owned(),
            slots: machine.slots,
            warm_runners: machine.warm_runners,
            draining: machine.draining,
            last_contact_at: machine.last_contact_at,
            agent_version: machine.agent_version.clone(),
            removed_at: machine.removed_at,
        }
    }

    pub(crate) fn into_record(self, disks: Vec<MachineDisk>) -> Result<Machine> {
        Ok(Machine {
            id: self.id,
            hostname: self.hostname,
            processor: self.processor,
            hardware_threads: self.hardware_threads,
            memory_bytes: self.memory_bytes,
            platform: parse_platform(&self.platform)?,
            virtualization: parse_virtualization(&self.virtualization)?,
            slots: self.slots,
            warm_runners: self.warm_runners,
            draining: self.draining,
            last_contact_at: self.last_contact_at,
            agent_version: self.agent_version,
            disks,
            removed_at: self.removed_at,
        })
    }
}
