//! Select the reported job disk and delegate every fit rule to the protocol.

use ciproxy_protocol::reports::HardwareReport;
use ciproxy_protocol::resources::Resources;

pub(crate) fn machine_resources(hardware: &HardwareReport) -> Resources {
    Resources::for_machine(
        hardware.threads,
        hardware.memory_bytes,
        hardware.disks.first().map_or(0, |disk| disk.capacity_bytes),
        hardware.kvm,
    )
}
