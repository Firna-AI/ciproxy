//! Hand-worked allocation boundaries protect free bytes and checked rounding.

use ciproxy_protocol::resources::{GIGABYTE, Resources, VmSize};

use crate::root_sizing::RootSizing;

#[test]
fn free_job_disk_adds_the_image_and_ext4_allowance_before_placement() {
    let sizing = RootSizing {
        image_root_bytes: 61_000_000_000,
    };
    for (free, expected) in [
        (1_000_000_000, 62_048_227_328),
        (20_000_000_000, 81_635_856_384),
        (60_000_000_000, 122_872_967_168),
    ] {
        assert_eq!(sizing.root_bytes(free), Some(expected), "free={free}");
        assert!(u128::from(expected - 61_000_000_000) * 97 >= u128::from(free + 16_777_216) * 100);
    }
    let size = sizing
        .vm_size(VmSize {
            vcpu: 2,
            memory_bytes: 8 * GIGABYTE,
            disk_bytes: 20 * GIGABYTE,
        })
        .unwrap();
    assert_eq!(
        Resources::for_machine(16, 64 * GIGABYTE, 1024 * GIGABYTE, true).concurrent_vms(size),
        6
    );
    assert_eq!(
        Resources::for_machine(4, 16 * GIGABYTE, 250 * GIGABYTE, true).concurrent_vms(size),
        1
    );
    assert_eq!(sizing.root_bytes(u64::MAX), None);
    assert_eq!(
        RootSizing {
            image_root_bytes: u64::MAX
        }
        .root_bytes(1),
        None
    );
}
