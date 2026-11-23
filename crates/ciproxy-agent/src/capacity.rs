//! Select the reported job disk and delegate every fit rule to the protocol.

use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::reports::HardwareReport;
use ciproxy_protocol::resources::{Resources, VmSize};

pub(crate) fn machine_resources(hardware: &HardwareReport, image_budget: u64) -> Resources {
    Resources::for_machine_with_image_budget(
        hardware.threads,
        hardware.memory_bytes,
        hardware.disks.first().map_or(0, |disk| disk.capacity_bytes),
        image_budget,
        hardware.virtualization.is_available(),
    )
}

/// Whether the disk has the real free space for a new VM of `size`.
///
/// A macOS VM is an APFS clone that grows as its job writes, so the job
/// disk count alone can promise space that running clones already took.
/// A Mac reports one disk, its area volume. A new macOS VM needs at least
/// the disk that it holds as real free space there. Linux VMs reserve their
/// complete root, so the count is the real space and this check passes.
pub(crate) fn real_space_fits(hardware: &HardwareReport, size: VmSize) -> bool {
    match hardware.platform {
        Platform::LinuxX64 => true,
        Platform::MacosArm64 => hardware
            .disks
            .first()
            .is_some_and(|disk| disk.free_bytes >= size.disk_bytes),
    }
}

#[cfg(test)]
#[path = "_tests_/real_space_tests.rs"]
mod real_space_tests;
