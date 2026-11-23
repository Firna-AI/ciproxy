//! Usage decode failures do not change the message that carries them.

use serde_json::error::Category;
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
    assert_messages(&malformed.to_string(), UsageReport::PointsDropped);
}

#[test]
fn usage_decode_keeps_points_when_the_steps_are_unreadable() {
    let mut expected = fixtures::job_usage();
    let mut malformed = serde_json::to_value(&expected).unwrap();
    malformed["steps"][0]["status"] = json!("not a step status");
    expected.steps.clear();
    assert_messages(&malformed.to_string(), UsageReport::StepsDropped(expected));
}

#[test]
fn usage_decode_drops_points_beyond_the_json_depth_limit() {
    let deep = format!("{}0{}", "[".repeat(200), "]".repeat(200));
    let usage = usage_with_raw_field("cpu_milli_vcpu", &deep);
    assert_messages(&usage, UsageReport::PointsDropped);

    for (packet, category, reason) in [
        (
            format!(
                r#"{{"usage":{usage},"type":"runner_exited","type":"runner_ready","runner_id":"first","exit":{{}}}}"#
            ),
            Category::Data,
            "duplicate field `type`",
        ),
        (
            format!(
                r#"{{"usage":{usage},"type":"runner_exited","runner_id":"first","runner_id":"second","exit":{{}}}}"#
            ),
            Category::Data,
            "duplicate field `runner_id`",
        ),
        (
            format!(
                r#"{{"usage":{usage},"type":"runner_exited","runner_id":"first","exit":{{"code":0,"code":1}}}}"#
            ),
            Category::Data,
            "duplicate field `code`",
        ),
        (
            format!(
                r#"{{"usage":{usage},"type":"runner_exited","runner_id":"\ud800","exit":{{}}}}"#
            ),
            Category::Syntax,
            "unexpected end of hex escape",
        ),
        (
            format!(r#"{{"usage":{usage},"type":"runner_exited","runner_id":"first"}}"#),
            Category::Data,
            "missing field `exit`",
        ),
        (
            format!(
                r#"{{"usage":null,"usa\u0067e":{usage},"type":"runner_exited","runner_id":"first","exit":{{}}}}"#
            ),
            Category::Data,
            "duplicate field `usage`",
        ),
        (
            r#"{"usage":"\q","type":"runner_exited","runner_id":"first","exit":{}}"#.into(),
            Category::Syntax,
            "invalid escape",
        ),
    ] {
        for error in [
            serde_json::from_str::<GuestToAgent>(&packet).unwrap_err(),
            serde_json::from_str::<AgentMessage>(&packet).unwrap_err(),
        ] {
            assert_eq!(error.classify(), category, "{error}");
            assert!(error.to_string().starts_with(reason), "{error}");
        }
    }
}

#[test]
fn usage_decode_keeps_points_when_a_step_exceeds_the_json_depth_limit() {
    let deep = format!("{}0{}", "[".repeat(200), "]".repeat(200));
    let steps = format!(r#"[{{"name":"Build","status":"ran","started_at_ms":{deep}}}]"#);
    assert_unreadable_steps(&steps);
}

#[test]
fn usage_decode_keeps_points_when_a_step_name_has_a_lone_surrogate() {
    assert_unreadable_steps(r#"[{"name":"\ud800","status":"skipped"}]"#);
}

fn assert_unreadable_steps(steps: &str) {
    let mut expected = fixtures::job_usage();
    expected.steps.clear();
    assert_messages(
        &usage_with_raw_field("steps", steps),
        UsageReport::StepsDropped(expected),
    );
}

fn usage_with_raw_field(field: &str, raw: &str) -> String {
    let mut usage = serde_json::to_value(fixtures::job_usage()).unwrap();
    usage[field] = json!("raw fixture field");
    usage.to_string().replace(r#""raw fixture field""#, raw)
}

fn assert_messages(usage: &str, expected: UsageReport) {
    let runner_id = fixtures::runner_id();
    let exit = RunnerExit { code: Some(7) };
    let runner_json = serde_json::to_string(&runner_id).unwrap();
    let exit_json = format!(
        r#"{{"usage":{usage},"runner_id":{runner_json},"exit":{{"code":7}},"type":"runner_exited"}}"#
    );
    let sample_json =
        format!(r#"{{"type":"usage_samples","runner_id":{runner_json},"usage":{usage}}}"#);
    assert_eq!(
        (
            serde_json::from_str::<GuestToAgent>(&exit_json).map_err(|error| error.to_string()),
            serde_json::from_str::<AgentMessage>(&exit_json).map_err(|error| error.to_string()),
            serde_json::from_str::<GuestToAgent>(&sample_json).map_err(|error| error.to_string()),
        ),
        (
            Ok(GuestToAgent::RunnerExited {
                runner_id: runner_id.clone(),
                exit,
                usage: Some(expected.clone()),
            }),
            Ok(AgentMessage::RunnerExited {
                runner_id: runner_id.clone(),
                exit,
                usage: Some(expected.clone()),
            }),
            Ok(GuestToAgent::UsageSamples {
                runner_id,
                usage: expected,
            }),
        ),
        "guest exit, agent exit, and sample report"
    );
}
