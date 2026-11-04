//! Disk capacity, complete fit, and held VM examples from job routing.

use crate::resources::{GIGABYTE, Resources, Slots, VmSize};

#[test]
fn machine_resources_keep_the_disk_reserve_and_existing_slots() {
    let examples = [
        (
            "mini PC",
            16,
            64 * GIGABYTE,
            256 * GIGABYTE,
            true,
            6,
            192 * GIGABYTE,
            64 * GIGABYTE,
        ),
        (
            "hardware minimum",
            4,
            16 * GIGABYTE,
            80 * GIGABYTE,
            true,
            1,
            60 * GIGABYTE,
            20 * GIGABYTE,
        ),
        (
            "no KVM",
            16,
            64 * GIGABYTE,
            256 * GIGABYTE,
            false,
            0,
            192 * GIGABYTE,
            64 * GIGABYTE,
        ),
        (
            "largest disk",
            16,
            64 * GIGABYTE,
            u64::MAX,
            true,
            6,
            13_835_058_055_282_163_711,
            4_611_686_018_427_387_904,
        ),
    ];
    for (name, threads, memory, disk, kvm, slots, job_disk, reserve) in examples {
        let resources = Resources::for_machine(threads, memory, disk, kvm);
        assert_eq!(resources.slots, Slots(slots), "{name}");
        assert_eq!(resources.job_disk_bytes, job_disk, "{name}");
        assert_eq!(disk - resources.job_disk_bytes, reserve, "{name}");
    }
}

#[test]
fn vm_fit_requires_slots_and_job_disk_on_one_machine() {
    let size = ciproxy_size();
    let examples = [
        (
            "hardware minimum",
            Resources::for_machine(4, 16 * GIGABYTE, 80 * GIGABYTE, true),
            true,
        ),
        (
            "disk one byte too small",
            Resources {
                slots: Slots(6),
                job_disk_bytes: 60 * GIGABYTE - 1,
            },
            false,
        ),
        (
            "slots too small",
            Resources {
                slots: Slots(0),
                job_disk_bytes: 192 * GIGABYTE,
            },
            false,
        ),
        (
            "no KVM",
            Resources::for_machine(16, 64 * GIGABYTE, 256 * GIGABYTE, false),
            false,
        ),
    ];
    for (name, resources, expected) in examples {
        assert_eq!(resources.fits(size), expected, "{name}");
    }
}

#[test]
fn default_sizes_use_the_smaller_slot_or_disk_concurrency() {
    let resources = mini_pc();
    let examples = [
        ("ciproxy", ciproxy_size(), 1, 6, 3, 3),
        (
            "ciproxy-4vcpu",
            VmSize {
                vcpu: 4,
                memory_bytes: 16 * GIGABYTE,
                disk_bytes: 80 * GIGABYTE,
            },
            2,
            3,
            2,
            2,
        ),
        (
            "ciproxy-8vcpu",
            VmSize {
                vcpu: 8,
                memory_bytes: 32 * GIGABYTE,
                disk_bytes: 120 * GIGABYTE,
            },
            4,
            1,
            1,
            1,
        ),
    ];
    for (name, size, slots, by_slots, by_disk, concurrent) in examples {
        assert_eq!(Slots::for_vm(size), Slots(slots), "{name}");
        assert_eq!(
            resources.slots.0 / Slots::for_vm(size).0,
            by_slots,
            "{name}"
        );
        assert_eq!(
            resources.job_disk_bytes / size.disk_bytes,
            by_disk,
            "{name}"
        );
        assert_eq!(resources.concurrent_vms(size), concurrent, "{name}");
    }

    let minimum = Resources::for_machine(4, 16 * GIGABYTE, 80 * GIGABYTE, true);
    assert_eq!(minimum.concurrent_vms(ciproxy_size()), 1);
    let cpu_limited = Resources::for_machine(4, 16 * GIGABYTE, 256 * GIGABYTE, true);
    assert_eq!(cpu_limited.concurrent_vms(ciproxy_size()), 1);
}

#[test]
fn warm_and_running_vms_hold_their_full_slots_and_disk() {
    let resources = mini_pc();
    let size = ciproxy_size();
    let two_warm = resources.available([size, size]);
    assert_eq!(
        two_warm,
        Resources {
            slots: Slots(4),
            job_disk_bytes: 72 * GIGABYTE
        }
    );
    assert_eq!(
        resources.job_disk_bytes - two_warm.job_disk_bytes,
        120 * GIGABYTE
    );
    assert!(two_warm.fits(size));
    assert_eq!(two_warm.concurrent_vms(size), 1);

    let third_vm = two_warm.available([size]);
    assert_eq!(
        third_vm,
        Resources {
            slots: Slots(3),
            job_disk_bytes: 12 * GIGABYTE
        }
    );
    assert!(!third_vm.fits(size));
    assert_eq!(third_vm.concurrent_vms(size), 0);
    assert!(resources.fits(size));

    let larger_running = VmSize {
        vcpu: 4,
        memory_bytes: 16 * GIGABYTE,
        disk_bytes: 80 * GIGABYTE,
    };
    let mixed = resources.available([size, larger_running]);
    assert_eq!(
        mixed,
        Resources {
            slots: Slots(3),
            job_disk_bytes: 52 * GIGABYTE
        }
    );
    assert!(!mixed.fits(size));
}

fn mini_pc() -> Resources {
    Resources::for_machine(16, 64 * GIGABYTE, 256 * GIGABYTE, true)
}

fn ciproxy_size() -> VmSize {
    VmSize {
        vcpu: 2,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes: 60 * GIGABYTE,
    }
}
