//! Linux procfs input fixtures for the usage readers.

use super::{Process, cpu, memory, oom_kills, worker_in_tree};

#[test]
fn reads_busy_cpu_without_idle_wait_steal_or_double_counted_guest_time() {
    let stat = include_str!("fixtures/stat");
    assert_eq!(cpu(stat, 100), Some((1_700_000, 2_000)));
    assert_eq!(cpu("cpu 100 20 30 10100 400", 100), None);
}

#[test]
fn reads_used_memory_from_available_instead_of_free() {
    let meminfo = include_str!("fixtures/meminfo");
    assert_eq!(memory(meminfo), Some(1_024));
    assert_eq!(memory("MemTotal: 4194304 kB\nMemFree: 65536 kB\n"), None);
}

#[test]
fn reads_the_oom_kill_counter() {
    let vmstat = include_str!("fixtures/vmstat");
    assert_eq!(oom_kills(vmstat), Some(12));
    assert_eq!(oom_kills("oom_kill_other 1\n"), None);
}

#[test]
fn detects_only_a_live_worker_below_the_runner() {
    let stats = include_str!("fixtures/processes")
        .lines()
        .map(|line| {
            let (pid, _) = line.split_once(' ').unwrap();
            (
                pid.parse().unwrap(),
                Process::from_stat(line.as_bytes()).unwrap(),
            )
        })
        .collect();
    assert!(worker_in_tree(&stats, 100));
    assert!(!worker_in_tree(&stats, 200));
}
