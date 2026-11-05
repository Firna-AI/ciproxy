//! Select the reported job disk and delegate every fit rule to the protocol.

use ciproxy_protocol::reports::HardwareReport;
use ciproxy_protocol::resources::Resources;

pub(crate) fn machine_resources(hardware: &HardwareReport, image_budget: u64) -> Resources {
    Resources::for_machine_with_image_budget(
        hardware.threads,
        hardware.memory_bytes,
        hardware.disks.first().map_or(0, |disk| disk.capacity_bytes),
        image_budget,
        hardware.kvm,
    )
}
