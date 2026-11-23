//! The step record format, timing, and limits.

use std::io::ErrorKind;

use ciproxy_protocol::usage::{JobStep, JobStepTiming};

use super::Steps;

const START: i64 = 1_709_164_800_000;

#[test]
fn pinned_worker_log_keeps_job_steps_and_incomplete_timing() {
    let mut steps = Steps::default();
    for line in include_str!("fixtures/worker-v2.338.2.txt").lines() {
        steps.observe(line).unwrap();
    }

    assert_eq!(
        steps.snapshot(),
        [
            ran("Run printf %s \"smoke\"", 1, Some(2)),
            skipped("Skip smoke"),
            ran("Composite's outer step", 3, Some(5)),
            ran("Post Run actions/checkout@v5", 6, Some(6)),
            ran("Interrupted step", 7, None),
        ]
    );
}

#[test]
fn rejects_malformed_step_records() {
    for line in [
        "[2024-02-30 00:00:00Z INFO StepsRunner] Processing step: DisplayName='Bad date'",
        "[2024-02-29 00:00:00Z INFO StepsRunner] Processing step: DisplayName='Unclosed name",
        "[2024-02-29 00:00:00Z INFO StepsRunner] Step result: Failed",
    ] {
        let mut steps = Steps::default();
        assert_eq!(
            steps.observe(line).map_err(|error| error.kind()),
            Err(ErrorKind::InvalidData),
            "malformed record: {line}"
        );
    }
}

#[test]
fn keeps_first_thousand_steps_and_cuts_names_at_character_boundaries() {
    let mut steps = Steps::default();
    let long_name = "é".repeat(300);
    for index in 0..1_002 {
        steps
            .observe(
                &format!(
                    "[2024-02-29 00:00:00Z INFO StepsRunner] Processing step: DisplayName='{index:04} {long_name}'"
                ),
            )
            .unwrap();
        steps
            .observe(
                "[2024-02-29 00:00:00Z INFO StepsRunner] Skipping step due to condition evaluation.",
            )
            .unwrap();
    }

    let reported = steps.snapshot();
    assert_eq!(reported.len(), 1_000);
    for (index, step) in reported.iter().enumerate() {
        assert_eq!(step, &skipped(&format!("{index:04} {}", "é".repeat(251))));
    }
}

fn ran(name: &str, start: i64, end: Option<i64>) -> JobStep {
    JobStep {
        name: name.into(),
        timing: JobStepTiming::Ran {
            started_at_ms: Some(START + start * 1_000),
            ended_at_ms: end.map(|second| START + second * 1_000),
        },
    }
}

fn skipped(name: &str) -> JobStep {
    JobStep {
        name: name.into(),
        timing: JobStepTiming::Skipped,
    }
}
