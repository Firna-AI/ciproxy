//! Usage decode failures do not change the message that carries them.

use serde_json::{Value, json};

use crate::agent::AgentMessage;
use crate::guest::GuestToAgent;
use crate::runners::RunnerExit;
use crate::usage::UsageReport;

use super::fixtures;

#[test]
fn usage_decode_drops_points_and_steps_when_the_points_are_unreadable() {
    let mut malformed = serde_json::to_value(fixtures::job_usage()).unwrap();
    malformed["cpu_milli_vcpu"] = json!([serde_json::from_str::<Value>("1e400").unwrap()]);
    assert_messages(malformed, UsageReport::PointsDropped);
}

#[test]
fn usage_decode_keeps_points_when_the_steps_are_unreadable() {
    let mut expected = fixtures::job_usage();
    let mut malformed = serde_json::to_value(&expected).unwrap();
    malformed["steps"][0]["status"] = json!("not a step status");
    expected.steps.clear();
    assert_messages(malformed, UsageReport::StepsDropped(expected));
}

fn assert_messages(usage: Value, expected: UsageReport) {
    let runner_id = fixtures::runner_id();
    let exit = RunnerExit { code: Some(7) };
    let exit_json = json!({
        "type": "runner_exited", "runner_id": runner_id, "exit": exit, "usage": usage,
    })
    .to_string();
    assert_eq!(
        serde_json::from_str::<GuestToAgent>(&exit_json).unwrap(),
        GuestToAgent::RunnerExited {
            runner_id: runner_id.clone(),
            exit,
            usage: Some(expected.clone()),
        }
    );
    assert_eq!(
        serde_json::from_str::<AgentMessage>(&exit_json).unwrap(),
        AgentMessage::RunnerExited {
            runner_id: runner_id.clone(),
            exit,
            usage: Some(expected.clone()),
        }
    );

    let sample_json = json!({
        "type": "usage_samples", "runner_id": runner_id, "usage": usage,
    })
    .to_string();
    assert_eq!(
        serde_json::from_str::<GuestToAgent>(&sample_json).unwrap(),
        GuestToAgent::UsageSamples {
            runner_id,
            usage: expected,
        }
    );
}
