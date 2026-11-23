//! Shared usage validation at the boundary before storage.

use chrono::DateTime;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};

use super::UsageLimits;

const START_MS: i64 = 1_700_000_000_000;
const ARRIVAL_MS: i64 = 1_700_000_600_000;

fn limits() -> UsageLimits {
    UsageLimits {
        vcpus: 2,
        memory_bytes: 8_388_608,
        started_at: DateTime::from_timestamp_millis(START_MS).unwrap(),
        received_at: DateTime::from_timestamp_millis(ARRIVAL_MS).unwrap(),
    }
}

fn step(name: impl Into<String>) -> JobStep {
    JobStep {
        name: name.into(),
        timing: JobStepTiming::Skipped,
    }
}

fn usage() -> JobUsage {
    JobUsage {
        platform: Platform::LinuxX64,
        first_point_at_ms: START_MS,
        point_seconds: 2,
        cpu_milli_vcpu: vec![2000],
        peak_memory_mib: vec![8],
        oom_kills: Some(vec![0]),
        steps: vec![step("Build")],
    }
}

#[test]
fn point_structure_errors_drop_the_whole_report() {
    let at_limit = JobUsage {
        cpu_milli_vcpu: vec![2000; 3600],
        peak_memory_mib: vec![8; 3600],
        oom_kills: Some(vec![0; 3600]),
        point_seconds: 1,
        ..usage()
    };
    let long_life = UsageLimits {
        received_at: DateTime::from_timestamp_millis(START_MS + 3_600_000).unwrap(),
        ..limits()
    };
    assert_eq!(
        long_life
            .prepare(at_limit, true)
            .unwrap()
            .usage
            .cpu_milli_vcpu
            .len(),
        3600
    );
    let cases = [
        (
            "point count",
            JobUsage {
                cpu_milli_vcpu: vec![2000; 3601],
                peak_memory_mib: vec![8; 3601],
                oom_kills: Some(vec![0; 3601]),
                point_seconds: 1,
                ..usage()
            },
        ),
        (
            "memory length",
            JobUsage {
                peak_memory_mib: vec![],
                ..usage()
            },
        ),
        (
            "kill count length",
            JobUsage {
                oom_kills: Some(vec![]),
                ..usage()
            },
        ),
        (
            "zero point width",
            JobUsage {
                point_seconds: 0,
                ..usage()
            },
        ),
        (
            "point width not a power of two",
            JobUsage {
                point_seconds: 3,
                ..usage()
            },
        ),
        (
            "no Linux kill counts",
            JobUsage {
                oom_kills: None,
                ..usage()
            },
        ),
        (
            "macOS with kill counts",
            JobUsage {
                platform: Platform::MacosArm64,
                ..usage()
            },
        ),
    ];
    for (case, report) in cases {
        assert_eq!(long_life.prepare(report, true).map(|_| ()), None, "{case}");
    }
}

#[test]
fn points_must_fit_the_byte_precise_vm_allocation() {
    assert_eq!(limits().prepare(usage(), true).unwrap().usage, usage());
    let over_cpu = JobUsage {
        cpu_milli_vcpu: vec![2001],
        ..usage()
    };
    assert_eq!(limits().prepare(over_cpu, true).map(|_| ()), None);
    let below_eight_mib = UsageLimits {
        memory_bytes: 8_388_607,
        ..limits()
    };
    assert_eq!(below_eight_mib.prepare(usage(), true).map(|_| ()), None);
}

#[test]
fn point_window_allows_exactly_five_minutes_of_clock_skew() {
    for (first_point_at_ms, accepted) in [
        (START_MS - 300_001, false),
        (START_MS - 300_000, true),
        (ARRIVAL_MS + 298_000, true),
        (ARRIVAL_MS + 298_001, false),
    ] {
        let report = JobUsage {
            first_point_at_ms,
            ..usage()
        };
        assert_eq!(
            limits().prepare(report, false).is_some(),
            accepted,
            "offset from runner start: {}",
            first_point_at_ms - START_MS
        );
    }
}

#[test]
fn step_limits_discard_only_steps() {
    let at_limit = JobUsage {
        steps: vec![step("é".repeat(256)); 1000],
        ..usage()
    };
    let saved = limits().prepare(at_limit, true).unwrap();
    assert_eq!(saved.usage.steps.len(), 1000);
    assert_eq!(saved.usage.steps[0], step("é".repeat(256)));
    let timed_step = |started_at_ms, ended_at_ms| JobStep {
        timing: JobStepTiming::Ran {
            started_at_ms,
            ended_at_ms,
        },
        ..step("Build")
    };
    let bad_steps = [
        vec![step("Build"); 1001],
        vec![step("Run ".to_owned() + &"é".repeat(253))],
        vec![step("Build\0")],
        vec![timed_step(Some(START_MS - 300_001), None)],
        vec![timed_step(None, Some(ARRIVAL_MS + 300_001))],
    ];
    let expected = JobUsage {
        steps: vec![],
        ..usage()
    };
    for (index, steps) in bad_steps.into_iter().enumerate() {
        let saved = limits()
            .prepare(JobUsage { steps, ..usage() }, true)
            .unwrap();
        assert_eq!(saved.usage.steps.len(), 0, "step limit case {index}");
        assert_eq!(saved.usage, expected);
    }
    let edge_step = timed_step(Some(START_MS - 300_000), Some(ARRIVAL_MS + 300_000));
    let report = JobUsage {
        steps: vec![edge_step.clone(), timed_step(Some(START_MS), None)],
        ..usage()
    };
    assert_eq!(
        limits().prepare(report, true).unwrap().usage.steps,
        [edge_step, timed_step(Some(START_MS), None)]
    );
}

#[test]
fn run_names_keep_only_remote_action_references() {
    let cases = [
        ("Run npm test", "Run script"),
        ("Pre Run ./build", "Pre Run script"),
        ("Post Run cleanup", "Post Run script"),
        ("Running tests", "Running tests"),
        ("Run", "Run"),
        ("Run actions/checkout@v4", "Run actions/checkout@v4"),
        ("Run foo/bar@baz", "Run foo/bar@baz"),
        ("Pre Run 3owner/repo/path@v1", "Pre Run 3owner/repo/path@v1"),
        (
            "Post Run docker://alpine:3.20",
            "Post Run docker://alpine:3.20",
        ),
        (
            "Run docker://registry.example:5000/owner/image@sha256:ab12",
            "Run docker://registry.example:5000/owner/image@sha256:ab12",
        ),
        ("Run ./build@v1", "Run script"),
        ("Run -owner/repo@v1", "Run script"),
        ("Run owner/repo@", "Run script"),
        ("Run owner/repo@v1 --flag", "Run script"),
        ("Run docker://", "Run script"),
        ("Run docker://alpine:3.20 --flag", "Run script"),
    ];
    let report = JobUsage {
        steps: cases.iter().map(|(name, _)| step(*name)).collect(),
        ..usage()
    };
    let saved = limits().prepare(report, false).unwrap();
    for (saved, (name, expected)) in saved.usage.steps.iter().zip(cases) {
        assert_eq!(saved.name, expected, "{name}");
        assert_eq!(saved.timing, JobStepTiming::Skipped);
    }
    assert_eq!(saved.usage.steps.len(), cases.len());
}
