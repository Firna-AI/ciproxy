//! Disk capacity, complete fit, and held VM examples from job routing.

use crate::resources::{GIGABYTE, Resources, Slots, VmSize};

#[test]
fn machine_resources_keep_the_disk_reserve_and_existing_slots() {
    let examples = [
        (
            "mini PC",
            16,
            64 * GIGABYTE,
            512 * GIGABYTE,
            true,
            6,
            279 * GIGABYTE,
            233 * GIGABYTE,
        ),
        (
            "hardware minimum",
            4,
            16 * GIGABYTE,
            250 * GIGABYTE,
            true,
            1,
            82_500_000_000,
            167_500_000_000,
        ),
        (
            "no KVM",
            16,
            64 * GIGABYTE,
            512 * GIGABYTE,
            false,
            0,
            279 * GIGABYTE,
            233 * GIGABYTE,
        ),
        (
            "largest disk",
            16,
            64 * GIGABYTE,
            u64::MAX,
            true,
            6,
            13_835_057_950_282_163_711,
            4_611_686_123_427_387_904,
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
    let four_vcpu = VmSize {
        vcpu: 4,
        memory_bytes: 16 * GIGABYTE,
        disk_bytes: 80 * GIGABYTE,
    };
    let eight_vcpu = VmSize {
        vcpu: 8,
        memory_bytes: 32 * GIGABYTE,
        disk_bytes: 120 * GIGABYTE,
    };
    let examples = [
        (
            "hardware minimum",
            Resources::for_machine(4, 16 * GIGABYTE, 250 * GIGABYTE, true),
            size,
            true,
        ),
        (
            "disk one byte too small",
            Resources {
                slots: Slots(6),
                job_disk_bytes: 80 * GIGABYTE - 1,
            },
            size,
            false,
        ),
        (
            "slots too small",
            Resources {
                slots: Slots(0),
                job_disk_bytes: 279 * GIGABYTE,
            },
            size,
            false,
        ),
        (
            "no KVM",
            Resources::for_machine(16, 64 * GIGABYTE, 512 * GIGABYTE, false),
            size,
            false,
        ),
        (
            "ciproxy-4vcpu exact fit",
            Resources {
                slots: Slots(2),
                job_disk_bytes: 80 * GIGABYTE,
            },
            four_vcpu,
            true,
        ),
        (
            "ciproxy-4vcpu disk too small",
            Resources {
                slots: Slots(2),
                job_disk_bytes: 80 * GIGABYTE - 1,
            },
            four_vcpu,
            false,
        ),
        (
            "ciproxy-4vcpu slots too small",
            Resources {
                slots: Slots(1),
                job_disk_bytes: 80 * GIGABYTE,
            },
            four_vcpu,
            false,
        ),
        (
            "ciproxy-8vcpu exact fit",
            Resources {
                slots: Slots(4),
                job_disk_bytes: 120 * GIGABYTE,
            },
            eight_vcpu,
            true,
        ),
        (
            "ciproxy-8vcpu disk too small",
            Resources {
                slots: Slots(4),
                job_disk_bytes: 80 * GIGABYTE,
            },
            eight_vcpu,
            false,
        ),
        (
            "ciproxy-8vcpu slots too small",
            Resources {
                slots: Slots(3),
                job_disk_bytes: 120 * GIGABYTE,
            },
            eight_vcpu,
            false,
        ),
    ];
    for (name, resources, size, expected) in examples {
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
            3,
            3,
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
            2,
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

    let minimum = Resources::for_machine(4, 16 * GIGABYTE, 250 * GIGABYTE, true);
    assert_eq!(minimum.concurrent_vms(ciproxy_size()), 1);
    let cpu_limited = Resources::for_machine(4, 16 * GIGABYTE, 512 * GIGABYTE, true);
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
            job_disk_bytes: 119 * GIGABYTE
        }
    );
    assert_eq!(
        resources.job_disk_bytes - two_warm.job_disk_bytes,
        160 * GIGABYTE
    );
    assert!(two_warm.fits(size));
    assert_eq!(two_warm.concurrent_vms(size), 1);

    let third_vm = two_warm.available([size]);
    assert_eq!(
        third_vm,
        Resources {
            slots: Slots(3),
            job_disk_bytes: 39 * GIGABYTE
        }
    );
    assert!(!third_vm.fits(size));
    assert_eq!(third_vm.concurrent_vms(size), 0);
    assert!(resources.fits(size));

    let larger_running = VmSize {
        vcpu: 4,
        memory_bytes: 16 * GIGABYTE,
        disk_bytes: 120 * GIGABYTE,
    };
    let mixed = resources.available([size, larger_running]);
    assert_eq!(
        mixed,
        Resources {
            slots: Slots(3),
            job_disk_bytes: 79 * GIGABYTE
        }
    );
    assert!(!mixed.fits(size));
}

fn mini_pc() -> Resources {
    Resources::for_machine(16, 64 * GIGABYTE, 512 * GIGABYTE, true)
}

fn ciproxy_size() -> VmSize {
    VmSize {
        vcpu: 2,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes: 80 * GIGABYTE,
    }
}
