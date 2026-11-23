//! Check the macOS conversions without Mach calls.

use std::time::Duration;

use super::super::CpuReading;
use super::memory_mib;

#[test]
fn host_tick_share_counts_all_busy_states_across_vcpus() {
    let previous = CpuReading::HostTicks {
        busy: [1_000, 2_000, 3_000],
        idle: 4_000,
    };
    let next = CpuReading::HostTicks {
        busy: [1_040, 2_012, 3_024],
        idle: 4_044,
    };
    assert_eq!(
        next.since(previous, Duration::from_secs(1), 4_000),
        Some(2_533)
    );
}

#[test]
fn host_tick_share_survives_counter_rollover() {
    let previous = CpuReading::HostTicks {
        busy: [u32::MAX - 9, 100, 200],
        idle: u32::MAX - 4,
    };
    let next = CpuReading::HostTicks {
        busy: [10, 110, 210],
        idle: 35,
    };
    assert_eq!(
        next.since(previous, Duration::from_secs(1), 2_000),
        Some(1_000)
    );
}

#[test]
fn host_tick_share_is_unavailable_when_counters_do_not_advance() {
    let reading = CpuReading::HostTicks {
        busy: [10, 20, 30],
        idle: 40,
    };
    assert_eq!(reading.since(reading, Duration::from_secs(1), 4_000), None);
}

#[test]
fn memory_sums_app_wired_and_compressed_pages_before_rounding() {
    assert_eq!(memory_mib(124, 37, 90, 45, 16_384), Some(3));
}

#[test]
fn memory_does_not_subtract_purgeable_pages_from_wired_or_compressed_pages() {
    assert_eq!(memory_mib(20, 24, 32, 32, 16_384), Some(1));
}
