//! Boat exits keep their lifecycle and the figures that can be read.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_protocol::usage::JobUsage;
use ciproxy_store_interface::job_usage::StoredJobUsage;
use serde_json::json;

use super::overflow_usage_support::{Callback, usage};
use super::support::at;

#[tokio::test]
async fn boat_keeps_the_exit_and_latest_samples_when_points_do_not_decode() {
    assert_unreadable_points(&usage_with_raw_field(
        "cpu_milli_vcpu",
        r#"["not a number"]"#,
    ))
    .await;
}

#[tokio::test]
async fn boat_drops_points_beyond_the_json_depth_limit() {
    let deep = format!("{}0{}", "[".repeat(200), "]".repeat(200));
    assert_unreadable_points(&usage_with_raw_field("cpu_milli_vcpu", &deep)).await;
}

#[tokio::test]
async fn boat_drops_points_with_a_lone_surrogate() {
    assert_unreadable_points(&usage_with_raw_field("cpu_milli_vcpu", r#"["\ud800"]"#)).await;
}

async fn assert_unreadable_points(bad_points: &str) {
    let callback = Callback::new().await;
    let sample_arrival = at() + Duration::seconds(1);
    callback.set_time(sample_arrival);
    let runner = callback.runner;
    let sample =
        format!(r#"{{"type":"usage_samples","runner_id":"{runner}","usage":{bad_points}}}"#);
    let sample_status = callback.post_raw(sample.into_bytes()).await;
    assert_eq!(callback.saved_usage(), None);
    let mut latest = usage();
    latest.steps.clear();
    assert_eq!(
        callback.sample(latest.clone()).await,
        StatusCode::NO_CONTENT
    );

    let exit_arrival = at() + Duration::seconds(2);
    callback.set_time(exit_arrival);
    let exit = format!(
        r#"{{"type":"runner_exited","runner_id":"{runner}","exit":{{"code":0}},"usage":{bad_points}}}"#
    );
    let exit_status = callback.post_raw(exit.into_bytes()).await;
    assert_eq!(
        (sample_status, exit_status),
        (StatusCode::UNPROCESSABLE_ENTITY, StatusCode::NO_CONTENT),
        "sample and exit response statuses"
    );
    assert_eq!(callback.delivery().reported_at, Some(exit_arrival));
    assert_eq!(callback.delivery().ends_at, Some(exit_arrival));
    assert_eq!(callback.harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(
        callback.saved_usage(),
        Some(StoredJobUsage {
            usage: latest,
            is_final: false,
            received_at: sample_arrival,
        })
    );
}

#[tokio::test]
async fn boat_keeps_the_exit_and_final_points_when_steps_do_not_decode() {
    let mut bad_steps = serde_json::to_value(usage()).unwrap();
    bad_steps["steps"][0]["status"] = json!("not a step status");
    assert_unreadable_steps(&bad_steps.to_string()).await;
}

#[tokio::test]
async fn boat_keeps_final_points_when_a_step_exceeds_the_json_depth_limit() {
    let deep = format!("{}0{}", "[".repeat(200), "]".repeat(200));
    let steps = format!(r#"[{{"name":"Build","status":"ran","started_at_ms":{deep}}}]"#);
    assert_unreadable_steps(&usage_with_raw_field("steps", &steps)).await;
}

#[tokio::test]
async fn boat_keeps_final_points_when_a_step_name_has_a_lone_surrogate() {
    let steps = r#"[{"name":"\ud800","status":"skipped"}]"#;
    assert_unreadable_steps(&usage_with_raw_field("steps", steps)).await;
}

async fn assert_unreadable_steps(bad_steps: &str) {
    let callback = Callback::new().await;
    let arrival = at() + Duration::seconds(1);
    callback.set_time(arrival);
    let runner = callback.runner;
    let exit = format!(
        r#"{{"type":"runner_exited","runner_id":"{runner}","exit":{{"code":0}},"usage":{bad_steps}}}"#
    );
    assert_eq!(
        callback.post_raw(exit.into_bytes()).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(callback.delivery().reported_at, Some(arrival));
    assert_eq!(callback.delivery().ends_at, Some(arrival));
    assert_eq!(callback.harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(
        callback.saved_usage(),
        Some(StoredJobUsage {
            usage: JobUsage {
                steps: vec![],
                ..usage()
            },
            is_final: true,
            received_at: arrival,
        })
    );
}

fn usage_with_raw_field(field: &str, raw: &str) -> String {
    let mut usage = serde_json::to_value(usage()).unwrap();
    usage[field] = json!("raw fixture field");
    usage.to_string().replace(r#""raw fixture field""#, raw)
}
