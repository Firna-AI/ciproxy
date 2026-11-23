//! Hand-worked allocation boundaries protect free bytes and checked rounding.

use ciproxy_protocol::resources::{GIGABYTE, Resources, VmSize};

use crate::root_sizing::RootSizing;

#[test]
fn free_job_disk_adds_the_image_and_ext4_allowance_before_placement() {
    let sizing = RootSizing::default();
    for (free, expected) in [
        (1_000_000_000, 61_311_987_712),
        (20_000_000_000, 80_899_612_672),
        (60_000_000_000, 122_136_727_552),
    ] {
        assert_eq!(sizing.root_bytes(free), Some(expected), "free={free}");
        assert!(
            u128::from(expected - sizing.image_root_bytes) * 97
                >= u128::from(free + 16_777_216) * 100
        );
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
    let available = Resources::for_machine(4, 16 * GIGABYTE, 250 * GIGABYTE, true)
        .available([size])
        .job_disk_bytes;
    assert_eq!(available, 1_600_387_328);
    assert!(available >= GIGABYTE);
    assert_eq!(sizing.root_bytes(u64::MAX), None);
    assert_eq!(
        RootSizing {
            image_root_bytes: u64::MAX
        }
        .root_bytes(1),
        None
    );
}

mod root_rule {
    use ciproxy_protocol::identifiers::ImageId;
    use ciproxy_protocol::images::ImageMetadata;
    use ciproxy_protocol::platform::Platform;
    use ciproxy_protocol::resources::{GIGABYTE, VmSize};

    use crate::root_sizing::{MACOS_ROOT_MARGIN_BYTES, RootRule, RootSizing};

    /// The standard macOS release: a 200 GB disk with 79 GB written.
    fn macos_image() -> ImageMetadata {
        ImageMetadata {
            image_id: ImageId("macos-26.7.3-1".to_owned()),
            root_disk_bytes: 200 * GIGABYTE,
            free_job_bytes: Some(121 * GIGABYTE),
        }
    }

    fn label(disk_bytes: u64) -> VmSize {
        VmSize {
            vcpu: 2,
            memory_bytes: 8 * GIGABYTE,
            disk_bytes,
        }
    }

    #[test]
    fn a_macos_vm_holds_its_label_disk_and_a_margin_not_the_image() {
        let root = RootRule::for_platform(
            Platform::MacosArm64,
            Some(&macos_image()),
            RootSizing::default(),
        );

        let size = root.vm_size(label(20 * GIGABYTE)).expect("fits the image");

        assert_eq!(size.disk_bytes, 20 * GIGABYTE + MACOS_ROOT_MARGIN_BYTES);
        assert_eq!(size.vcpu, 2);
        assert_eq!(size.memory_bytes, 8 * GIGABYTE);
        assert_eq!(
            root.root_bytes(121 * GIGABYTE),
            Some(121 * GIGABYTE + MACOS_ROOT_MARGIN_BYTES)
        );
    }

    #[test]
    fn a_macos_label_above_the_free_space_of_its_image_gets_no_root() {
        let root = RootRule::for_platform(
            Platform::MacosArm64,
            Some(&macos_image()),
            RootSizing::default(),
        );

        assert_eq!(root.root_bytes(121 * GIGABYTE + 1), None);
        assert_eq!(root.vm_size(label(200 * GIGABYTE)), None);
    }

    #[test]
    fn a_linux_vm_keeps_the_ext4_root_of_its_image() {
        let image = ImageMetadata {
            image_id: ImageId("ubuntu-24.04-1".to_owned()),
            root_disk_bytes: 60_263_759_872,
            free_job_bytes: None,
        };
        let sizing = RootSizing::for_image(&image);

        for selected in [Some(&image), None] {
            let root = RootRule::for_platform(Platform::LinuxX64, selected, sizing);
            assert_eq!(root, RootRule::Ext4(sizing));
            assert_eq!(
                root.root_bytes(20 * GIGABYTE),
                sizing.root_bytes(20 * GIGABYTE)
            );
        }
    }
}
