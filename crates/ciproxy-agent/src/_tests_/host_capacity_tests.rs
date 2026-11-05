//! The underlying disk is reported once. The mounted XFS size is not reserved twice.

use std::sync::Arc;

use unimock::{MockFn, Unimock, matching};

use crate::area::AreaFilesMock;
use crate::boundaries::Host;
use crate::capacity::machine_resources;
use crate::hardware::LinuxHost;
use crate::host_fixtures::inventory;
use crate::host_system::HostSystemMock;

#[test]
fn minimum_host_reports_sixty_gb_job_disk_and_missing_kvm_reports_zero_slots() {
    for (kvm, host, physical_free, area_free, capacity, free, job_disk, slots) in [
        (
            true,
            80_000_000_000,
            65_000_000_000,
            Some(50_000_000_000),
            80_000_000_000,
            50_000_000_000,
            60_000_000_000,
            1,
        ),
        (
            true,
            160_000_000_000,
            150_000_000_000,
            Some(59_000_000_000),
            80_000_000_000,
            59_000_000_000,
            60_000_000_000,
            1,
        ),
        (
            true,
            80_000_000_000,
            20_000_000_000,
            Some(0),
            80_000_000_000,
            0,
            60_000_000_000,
            1,
        ),
        (
            false,
            80_000_000_000,
            5_000_000_000,
            Some(59_000_000_000),
            80_000_000_000,
            5_000_000_000,
            60_000_000_000,
            0,
        ),
        (
            true,
            40_000_000_000,
            35_000_000_000,
            Some(10_000_000_000),
            40_000_000_000,
            10_000_000_000,
            30_000_000_000,
            1,
        ),
        (
            true,
            80_000_000_000,
            65_000_000_000,
            None,
            80_000_000_000,
            0,
            60_000_000_000,
            1,
        ),
    ] {
        let mut current = inventory();
        current.host_bytes = host;
        current.free_bytes = physical_free;
        current.area_free = area_free;
        let mock = Arc::new(Unimock::new((
            HostSystemMock::read.some_call(matching!("/proc/cpuinfo"))
                .returns(Ok(b"processor\t: 0\nprocessor\t: 1\nprocessor\t: 2\nprocessor\t: 3\nmodel name: fixture\n".to_vec())),
            HostSystemMock::read.some_call(matching!("/proc/meminfo"))
                .returns(Ok(b"MemTotal: 16000000 kB\n".to_vec())),
            HostSystemMock::kvm.some_call(matching!()).returns(kvm),
            AreaFilesMock::inspect.some_call(matching!()).returns(Ok(current)),
        )));
        let hardware = LinuxHost {
            system: mock.clone(),
            area: mock.clone(),
        }
        .hardware()
        .unwrap();
        assert_eq!(hardware.disks[0].capacity_bytes, capacity);
        assert_eq!(hardware.disks[0].free_bytes, free);
        assert_eq!(machine_resources(&hardware).job_disk_bytes, job_disk);
        assert_eq!(machine_resources(&hardware).slots.0, slots);
    }
}
