//! The agent's line limit for guest exit reports.

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage, MAX_STEP_NAME_CHARS, MAX_STEPS};

use super::encode;

#[test]
fn oversized_step_list_does_not_drop_the_guest_exit_line() {
    let long = step("🟢".repeat(MAX_STEP_NAME_CHARS));
    let bytes = encode(exit_with(vec![long; MAX_STEPS])).unwrap();
    assert!(
        bytes.len() <= 1_048_576,
        "exit line exceeds the agent limit"
    );
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert_eq!(
        serde_json::from_slice::<GuestToAgent>(&bytes).unwrap(),
        exit_with(Vec::new())
    );

    let normal = exit_with(vec![step("Short name".into())]);
    let bytes = encode(normal.clone()).unwrap();
    assert_eq!(
        serde_json::from_slice::<GuestToAgent>(&bytes).unwrap(),
        normal
    );
}

fn step(name: String) -> JobStep {
    JobStep {
        name,
        timing: JobStepTiming::Ran {
            started_at_ms: Some(1_700_000_000_000),
            ended_at_ms: Some(1_700_000_001_000),
        },
    }
}

fn exit_with(steps: Vec<JobStep>) -> GuestToAgent {
    GuestToAgent::RunnerExited {
        runner_id: RunnerId("runner-7".into()),
        exit: RunnerExit { code: Some(7) },
        usage: Some(JobUsage {
            platform: Platform::LinuxX64,
            first_point_at_ms: 1_700_000_000_000,
            point_seconds: 1,
            cpu_milli_vcpu: vec![500],
            peak_memory_mib: vec![100],
            oom_kills: Some(vec![0]),
            steps,
        }),
    }
}
