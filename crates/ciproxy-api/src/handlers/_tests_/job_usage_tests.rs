//! Job and step figures from the stored points before chart merging.

use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_store_interface::vm_size;
use serde_json::{Value, json, to_value};

use super::measured;

const VM: VmSize = VmSize {
    vcpu: 4,
    memory_bytes: 8_000_000_000,
    disk_bytes: 60_000_000_000,
};

fn linux(cpu: Vec<u32>, memory: Vec<u32>, kills: Vec<u32>) -> JobUsage {
    JobUsage {
        platform: Platform::LinuxX64,
        first_point_at_ms: 0,
        point_seconds: 1,
        cpu_milli_vcpu: cpu,
        peak_memory_mib: memory,
        oom_kills: Some(kills),
        steps: Vec::new(),
    }
}

fn ran(name: &str, start: Option<i64>, end: Option<i64>) -> JobStep {
    JobStep {
        name: name.into(),
        timing: JobStepTiming::Ran {
            started_at_ms: start,
            ended_at_ms: end,
        },
    }
}

fn figures(cpu: u32, memory: &str, kills: Option<u32>) -> Value {
    json!({"average_cpu_milli_vcpu": cpu, "peak_memory_bytes": memory, "oom_kills": kills})
}

#[test]
fn step_figures_use_point_midpoints_and_include_setup_and_completion() {
    let mut usage = linux(
        vec![200, 400, 800, 1200, 1600, 2000],
        vec![128, 256, 180, 1024, 2048, 128],
        vec![0, 1, 0, 2, 0, 0],
    );
    usage.point_seconds = 2;
    usage.steps = vec![
        ran("Check settings", Some(2000), Some(3000)),
        ran("Run script", Some(3000), Some(6500)),
        JobStep {
            name: "Optional check".into(),
            timing: JobStepTiming::Skipped,
        },
        ran("Tests", Some(6500), Some(10100)),
    ];

    let view = to_value(measured(&usage, VM, None)).unwrap();
    assert_eq!(view["job"], figures(1033, "2147483648", Some(3)));
    assert_eq!(
        view["steps"],
        json!([
            {"number": 1, "name": "Set up job", "script": false, "skipped": false,
             "started_at": "1970-01-01T00:00:00Z", "ended_at": "1970-01-01T00:00:02Z",
             "duration_seconds": "2", "figures": figures(200, "134217728", Some(0))},
            {"number": 2, "name": "Check settings", "script": false, "skipped": false,
             "started_at": "1970-01-01T00:00:02Z", "ended_at": "1970-01-01T00:00:03Z",
             "duration_seconds": "1", "figures": null},
            {"number": 3, "name": "Run script", "script": true, "skipped": false,
             "started_at": "1970-01-01T00:00:03Z", "ended_at": "1970-01-01T00:00:06.500Z",
             "duration_seconds": "3.5", "figures": figures(600, "268435456", Some(1))},
            {"number": 4, "name": "Optional check", "script": false, "skipped": true,
             "started_at": null, "ended_at": null, "duration_seconds": null, "figures": null},
            {"number": 5, "name": "Tests", "script": false, "skipped": false,
             "started_at": "1970-01-01T00:00:06.500Z", "ended_at": "1970-01-01T00:00:10.100Z",
             "duration_seconds": "3.6", "figures": figures(1400, "2147483648", Some(2))},
            {"number": 6, "name": "Complete job", "script": false, "skipped": false,
             "started_at": "1970-01-01T00:00:10.100Z", "ended_at": "1970-01-01T00:00:12Z",
             "duration_seconds": "1.9", "figures": figures(2000, "134217728", Some(0))}
        ])
    );
}

#[test]
fn steps_outside_the_samples_or_without_both_times_keep_no_figures() {
    let mut usage = linux(vec![250; 4], vec![128; 4], vec![0; 4]);
    usage.steps = vec![
        ran("Outside", Some(-2000), Some(-1000)),
        ran("No start", None, Some(2000)),
        ran("No end", Some(2000), None),
    ];

    let view = to_value(measured(&usage, VM, None)).unwrap();
    assert_eq!(
        view["steps"],
        json!([
            {"number": 1, "name": "Set up job", "script": false, "skipped": false,
             "started_at": "1970-01-01T00:00:00Z", "ended_at": "1970-01-01T00:00:00Z",
             "duration_seconds": "0", "figures": null},
            {"number": 2, "name": "Outside", "script": false, "skipped": false,
             "started_at": "1969-12-31T23:59:58Z", "ended_at": "1969-12-31T23:59:59Z",
             "duration_seconds": "1", "figures": null},
            {"number": 3, "name": "No start", "script": false, "skipped": false,
             "started_at": null, "ended_at": "1970-01-01T00:00:02Z",
             "duration_seconds": null, "figures": null},
            {"number": 4, "name": "No end", "script": false, "skipped": false,
             "started_at": "1970-01-01T00:00:02Z", "ended_at": null,
             "duration_seconds": null, "figures": null},
            {"number": 5, "name": "Complete job", "script": false, "skipped": false,
             "started_at": null, "ended_at": "1970-01-01T00:00:04Z",
             "duration_seconds": null, "figures": null}
        ])
    );
}

#[test]
fn job_without_steps_keeps_figures_and_chart_without_macos_kill_counts() {
    let usage = JobUsage {
        platform: Platform::MacosArm64,
        first_point_at_ms: 0,
        point_seconds: 1,
        cpu_milli_vcpu: vec![100, 900],
        peak_memory_mib: vec![128, 256],
        oom_kills: None,
        steps: Vec::new(),
    };

    assert_eq!(
        to_value(measured(&usage, VM, None)).unwrap(),
        json!({
            "state": "measured", "job": figures(500, "268435456", None),
            "vm": {"vcpus": 4, "memory_bytes": "8000000000"}, "label_mark": null, "steps": [],
            "chart": {"started_at": "1970-01-01T00:00:00Z", "ended_at": "1970-01-01T00:00:02Z",
                      "point_seconds": 1, "points": [figures(100, "134217728", None),
                                                      figures(900, "268435456", None)]}
        })
    );
}

#[test]
fn caps_chart_at_600_points_and_keeps_raw_step_figures_and_the_tail() {
    let at_limit = linux(vec![100; 600], vec![64; 600], vec![0; 600]);
    let view = to_value(measured(&at_limit, VM, None)).unwrap();
    assert_eq!(
        view["chart"],
        json!({"started_at": "1970-01-01T00:00:00Z", "ended_at": "1970-01-01T00:10:00Z",
               "point_seconds": 1, "points": vec![figures(100, "67108864", Some(0)); 600]})
    );

    let mut usage = linux(vec![1; 2401], vec![64; 2401], vec![0; 2401]);
    usage.cpu_milli_vcpu[..8].copy_from_slice(&[0, 1, 0, 1, 0, 1, 0, 5]);
    usage.cpu_milli_vcpu[2400] = 999;
    usage.peak_memory_mib[6] = 256;
    usage.peak_memory_mib[2400] = 96;
    usage.oom_kills.as_mut().unwrap()[5..8].copy_from_slice(&[4, 0, 4]);
    usage.oom_kills.as_mut().unwrap()[2400] = 2;
    usage.steps = vec![ran("Short step", Some(6000), Some(7000))];

    let view = to_value(measured(&usage, VM, None)).unwrap();
    let mut expected = vec![figures(1, "67108864", Some(0)); 301];
    expected[0] = figures(1, "268435456", Some(8));
    expected[300] = figures(999, "100663296", Some(2));
    assert_eq!(
        view["chart"],
        json!({"started_at": "1970-01-01T00:00:00Z", "ended_at": "1970-01-01T00:40:01Z",
               "point_seconds": 8, "points": expected})
    );
    assert_eq!(
        view["steps"][1]["figures"],
        figures(0, "268435456", Some(0))
    );
    assert_eq!(view["job"], figures(1, "268435456", Some(10)));
}

#[test]
fn reports_only_label_bounds_smaller_than_the_actual_vm() {
    let usage = linux(vec![100], vec![64], vec![0]);
    for (vcpus, memory_gb, expected) in [
        (2, 8, json!({"vcpus": 2, "memory_bytes": null})),
        (4, 4, json!({"vcpus": null, "memory_bytes": "4000000000"})),
        (4, 8, Value::Null),
    ] {
        let label = vm_size::VmSize {
            vcpus,
            memory_gb,
            disk_gb: 20,
        };
        let view = to_value(measured(&usage, VM, Some(label))).unwrap();
        assert_eq!(view["state"], "measured");
        assert_eq!(view["label_mark"], expected, "{vcpus} vCPU, {memory_gb} GB");
    }
}
