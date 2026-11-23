//! A new macOS VM needs its held disk as real free space on the area volume.

use ciproxy_protocol::platform::{Platform, Virtualization};
use ciproxy_protocol::reports::{DiskReport, HardwareReport};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};

use super::real_space_fits;

fn hardware() -> HardwareReport {
    HardwareReport {
        platform: Platform::LinuxX64,
        processor: "Fixture CPU".to_owned(),
        threads: 16,
        memory_bytes: 64 * GIGABYTE,
        virtualization: Virtualization::Available,
        disks: vec![DiskReport {
            name: "area".to_owned(),
            capacity_bytes: 256 * GIGABYTE,
            free_bytes: 100 * GIGABYTE,
        }],
    }
}

fn size(disk_bytes: u64) -> VmSize {
    VmSize {
        vcpu: 4,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes,
    }
}

#[test]
fn a_mac_refuses_a_vm_that_its_real_free_space_cannot_hold() {
    let mut mac = hardware();
    mac.platform = Platform::MacosArm64;
    mac.disks[0].free_bytes = 30 * GIGABYTE;

    assert!(real_space_fits(&mac, size(24 * GIGABYTE)));
    assert!(real_space_fits(&mac, size(30 * GIGABYTE)));
    assert!(!real_space_fits(&mac, size(30 * GIGABYTE + 1)));

    mac.disks.clear();
    assert!(!real_space_fits(&mac, size(GIGABYTE)));
}

#[test]
fn linux_reserves_its_roots_so_the_real_free_space_is_not_checked() {
    let mut linux = hardware();
    linux.disks[0].free_bytes = 0;

    assert!(real_space_fits(&linux, size(100 * GIGABYTE)));
}
