//! Sample placement on a fake monotonic clock.

use std::time::Duration;

use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::usage::JobUsage;
use unimock::{MockFn, Unimock, matching};

use crate::usage::{CpuReading, Reading, SourceMock};

use super::Window;

const UTC_START_MS: i64 = 1_700_000_000_000;

#[test]
fn fills_one_skipped_slot_without_shifting_the_later_points() {
    let mut window = Window::new(Platform::LinuxX64);
    for (elapsed_ms, busy, idle, memory, kills) in [
        (0, 0, 0, 100, 0),
        (1_000, 2, 8, 200, 0),
        (2_500, 8, 22, 250, 1),
        (4_001, 12, 28, 400, 4),
        (5_000, 17, 33, 500, 4),
    ] {
        capture(&mut window, elapsed_ms, busy, idle, memory, kills);
    }

    assert_eq!(
        window.report(),
        Some(JobUsage {
            platform: Platform::LinuxX64,
            first_point_at_ms: 1_700_000_000_000,
            point_seconds: 1,
            cpu_milli_vcpu: vec![200, 300, 400, 400, 500],
            peak_memory_mib: vec![200, 250, 250, 400, 500],
            oom_kills: Some(vec![0, 1, 0, 3, 0]),
            steps: vec![],
        })
    );
}

#[test]
fn ends_the_run_at_a_two_second_stall() {
    let mut window = Window::new(Platform::LinuxX64);
    for (elapsed_ms, busy, idle, memory, kills) in [
        (0, 0, 0, 100, 0),
        (1_000, 2, 8, 110, 0),
        (3_000, 10, 20, 130, 9),
        (4_000, 14, 26, 140, 10),
        (5_000, 19, 31, 150, 10),
    ] {
        capture(&mut window, elapsed_ms, busy, idle, memory, kills);
    }

    assert_eq!(
        window.report(),
        Some(JobUsage {
            platform: Platform::LinuxX64,
            first_point_at_ms: 1_700_000_003_000,
            point_seconds: 1,
            cpu_milli_vcpu: vec![400, 500],
            peak_memory_mib: vec![140, 150],
            oom_kills: Some(vec![1, 0]),
            steps: vec![],
        })
    );
}

fn capture(window: &mut Window, elapsed_ms: u64, busy: u32, idle: u32, memory: u32, kills: u64) {
    let source = Unimock::new(SourceMock::read.some_call(matching!()).returns(Ok(Reading {
        at_ms: UTC_START_MS + elapsed_ms as i64,
        cpu: CpuReading::HostTicks {
            busy: [busy, 0, 0],
            idle,
        },
        max_cpu_milli_vcpu: 1_000,
        memory_mib: memory,
        oom_kills: kills,
    })));
    window.capture(&source, Duration::from_millis(elapsed_ms));
}
