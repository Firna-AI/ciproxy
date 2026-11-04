//! Hand-worked slot arithmetic examples from the routing requirements.

use crate::resources::{GIGABYTE, Slots, VmSize};

#[test]
fn machine_slots_reserve_a_quarter_and_round_down() {
    let examples = [
        ("AX42", 16, 64 * GIGABYTE, true, 6),
        ("minimum machine", 4, 16 * GIGABYTE, true, 1),
        ("CPU limits capacity", 8, 64 * GIGABYTE, true, 3),
        ("memory limits capacity", 16, 32 * GIGABYTE, true, 3),
        ("decimal GB boundary", 16, 32_000_000_000, true, 3),
        ("fractional reserve", 7, 19 * GIGABYTE, true, 1),
        ("fractional thread reserve", 5, 24 * GIGABYTE, true, 1),
        ("fraction below one slot", 2, 8 * GIGABYTE, true, 0),
        ("no KVM", 16, 64 * GIGABYTE, false, 0),
        ("no memory", 16, 0, true, 0),
        ("largest reports", u32::MAX, u64::MAX, true, 1_610_612_735),
    ];
    for (name, threads, memory, kvm, expected) in examples {
        assert_eq!(
            Slots::for_machine(threads, memory, kvm),
            Slots(expected),
            "{name}"
        );
    }
}

#[test]
fn vm_slots_cover_both_resources_and_round_up() {
    let examples = [
        ("ciproxy", 2, 8 * GIGABYTE, 1),
        ("ciproxy-4vcpu", 4, 16 * GIGABYTE, 2),
        ("ciproxy-8vcpu", 8, 32 * GIGABYTE, 4),
        ("CPU rounds up", 3, 8 * GIGABYTE, 2),
        ("odd CPU request", 5, 8 * GIGABYTE, 3),
        ("memory requires three", 2, 17 * GIGABYTE, 3),
        ("memory rounds up", 2, 8 * GIGABYTE + 1, 2),
        ("decimal GB rounds up", 2, 8_000_000_001, 2),
        ("memory dominates", 3, 25 * GIGABYTE, 4),
        ("CPU dominates", 9, 8 * GIGABYTE, 5),
        ("largest memory", 1, u64::MAX, 2_305_843_010),
    ];
    for (name, vcpu, memory_bytes, expected) in examples {
        let size = VmSize {
            vcpu,
            memory_bytes,
            disk_bytes: 60 * GIGABYTE,
        };
        assert_eq!(Slots::for_vm(size), Slots(expected), "{name}");
    }
}

#[test]
fn size_fits_one_machine_only_when_its_slots_cover_the_request() {
    let small = Slots::for_vm(VmSize {
        vcpu: 2,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes: 60 * GIGABYTE,
    });
    let large = Slots::for_vm(VmSize {
        vcpu: 8,
        memory_bytes: 32 * GIGABYTE,
        disk_bytes: 120 * GIGABYTE,
    });
    let machine = Slots::for_machine(4, 16 * GIGABYTE, true);
    assert!(machine.fits(small));
    assert!(!machine.fits(large));
}
