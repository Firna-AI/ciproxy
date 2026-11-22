//! Contact values that can write only observed machine columns.

use chrono::{DateTime, Utc};
use ciproxy_schema::machines::machines;
use ciproxy_store_interface::gateway::Contact;
use ciproxy_store_interface::machines::MachineDisk;
use diesel::prelude::*;

use crate::platform_text::{platform_text, virtualization_text};

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = machines)]
pub(crate) struct MachineObservation {
    hostname: String,
    processor: String,
    hardware_threads: i32,
    memory_bytes: i64,
    #[diesel(skip_update)]
    platform: &'static str,
    virtualization: &'static str,
    slots: i32,
    last_contact_at: DateTime<Utc>,
    agent_version: String,
}

impl MachineObservation {
    pub(crate) fn from_contact(contact: &Contact) -> Self {
        let report = &contact.report;
        Self {
            hostname: report.hostname.clone(),
            processor: report.hardware.processor.clone(),
            hardware_threads: i32::try_from(report.hardware.threads).unwrap_or(i32::MAX),
            memory_bytes: i64::try_from(report.hardware.memory_bytes).unwrap_or(i64::MAX),
            platform: platform_text(report.hardware.platform),
            virtualization: virtualization_text(report.hardware.virtualization),
            slots: i32::try_from(report.capacity.0).unwrap_or(i32::MAX),
            last_contact_at: contact.at,
            agent_version: report.agent_version.clone(),
        }
    }
}

pub(crate) fn observed_disks(contact: &Contact) -> Vec<MachineDisk> {
    contact
        .report
        .hardware
        .disks
        .iter()
        .map(|disk| MachineDisk {
            path: disk.name.clone(),
            total_bytes: i64::try_from(disk.capacity_bytes).unwrap_or(i64::MAX),
            free_bytes: i64::try_from(disk.free_bytes).unwrap_or(i64::MAX),
        })
        .collect()
}
