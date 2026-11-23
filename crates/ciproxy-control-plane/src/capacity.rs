//! Dashboard capacity uses the same prepared bytes as hosted root admission.

use ciproxy_agent::root_sizing::{RootRule, RootSizing};
use ciproxy_api::capacity::{Capacity, MachineCapacity};
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::gateway::MachineSnapshot;

pub(crate) struct ReportedCapacity;

impl MachineCapacity for ReportedCapacity {
    fn capacity(&self, snapshot: &MachineSnapshot) -> Capacity {
        match &snapshot.report {
            Some(report) => Capacity {
                slots: report.report.capacity,
                // A Mac has no Docker disks, as docker-disks.md says, so it
                // leases no bytes and sends no Docker report.
                job_disk_bytes: if snapshot.machine.platform == Platform::MacosArm64 {
                    Some(report.report.job_disk_bytes)
                } else {
                    report
                        .report
                        .docker_disks
                        .as_ref()
                        .measured()
                        .map(|docker| {
                            report
                                .report
                                .job_disk_bytes
                                .saturating_sub(docker.leased_bytes)
                        })
                },
                image_disk_bytes: report
                    .report
                    .images
                    .selected()
                    .map(|image| image.root_disk_bytes),
            },
            None => Capacity {
                slots: Slots(0),
                job_disk_bytes: None,
                image_disk_bytes: None,
            },
        }
    }

    fn required_root_bytes(&self, snapshot: &MachineSnapshot, requested: u64) -> Option<u64> {
        let image = snapshot.report.as_ref()?.report.images.selected()?;
        RootRule::for_platform(
            snapshot.machine.platform,
            Some(image),
            RootSizing::for_image(image),
        )
        .root_bytes(requested)
    }
}

#[cfg(test)]
#[path = "_tests_/capacity_tests.rs"]
mod capacity_tests;
