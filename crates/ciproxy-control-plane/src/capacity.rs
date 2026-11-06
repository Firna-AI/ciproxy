//! Dashboard capacity uses the same prepared bytes as hosted root admission.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_api::capacity::{Capacity, MachineCapacity};
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::gateway::MachineSnapshot;

pub(crate) struct ReportedCapacity;

impl MachineCapacity for ReportedCapacity {
    fn capacity(&self, snapshot: &MachineSnapshot) -> Capacity {
        match &snapshot.report {
            Some(report) => Capacity {
                slots: report.capacity,
                job_disk_bytes: Some(
                    report
                        .job_disk_bytes
                        .saturating_sub(report.docker_disks.leased_bytes),
                ),
                image_disk_bytes: report.images.selected().map(|image| image.root_disk_bytes),
            },
            None => Capacity {
                slots: Slots(0),
                job_disk_bytes: None,
                image_disk_bytes: None,
            },
        }
    }

    fn required_root_bytes(&self, snapshot: &MachineSnapshot, requested: u64) -> Option<u64> {
        RootSizing::for_image(snapshot.report.as_ref()?.images.selected()?).root_bytes(requested)
    }
}

#[cfg(test)]
#[path = "_tests_/capacity_tests.rs"]
mod capacity_tests;
