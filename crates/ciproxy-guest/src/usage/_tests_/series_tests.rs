//! Jobs on a fake one-second clock, including multiple point merges.

use ciproxy_protocol::platform::Platform;

use super::{Sample, Series};

#[test]
fn merges_neighbors_at_the_limit_and_keeps_the_partial_tail() {
    let mut clock_ms = 1_700_000_000_123;
    let mut series = Series::new(clock_ms);
    let mut values = [(10, 256, 0), (11, 768, 2), (10, 512, 1), (13, 1_024, 4)]
        .into_iter()
        .cycle();
    let mut advance_to = |end_ms, series: &mut Series| {
        while clock_ms < end_ms {
            clock_ms += 1_000;
            let (cpu, memory, kills) = values.next().unwrap();
            series.push(Sample {
                cpu_milli_vcpu: cpu,
                memory_mib: memory,
                oom_kills: kills,
            });
        }
    };

    advance_to(1_700_003_600_123, &mut series);
    let usage = series.report(Platform::LinuxX64);
    assert_eq!(usage.point_seconds, 1);
    assert_eq!(usage.cpu_milli_vcpu.len(), 3_600);

    advance_to(1_700_003_601_123, &mut series);
    let usage = series.report(Platform::LinuxX64);
    assert_eq!(usage.point_seconds, 2);
    assert_eq!(usage.cpu_milli_vcpu.len(), 1_801);
    assert_eq!(usage.cpu_milli_vcpu[..2], [10, 11]);
    assert_eq!(usage.peak_memory_mib[..2], [768, 1_024]);
    assert_eq!(usage.oom_kills.unwrap()[..2], [2, 5]);

    advance_to(1_700_007_201_123, &mut series);
    let usage = series.report(Platform::LinuxX64);
    assert_eq!(usage.validate(), Ok(()));
    assert_eq!(usage.first_point_at_ms, 1_700_000_000_123);
    assert_eq!(usage.point_seconds, 4);
    assert_eq!(series.seconds(), 7_201);
    assert_eq!(usage.cpu_milli_vcpu, [vec![11; 1_800], vec![10]].concat());
    assert_eq!(
        usage.peak_memory_mib,
        [vec![1_024; 1_800], vec![256]].concat()
    );
    assert_eq!(usage.oom_kills, Some([vec![7; 1_800], vec![0]].concat()));
    assert_eq!(usage.steps, vec![]);
}
