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
    let callback = Callback::new().await;
    let sample_arrival = at() + Duration::seconds(1);
    callback.set_time(sample_arrival);
    let mut bad_points = serde_json::to_value(usage()).unwrap();
    bad_points["cpu_milli_vcpu"] = json!(["not a number"]);
    let sample = json!({
        "type": "usage_samples", "runner_id": callback.runner, "usage": bad_points,
    });
    assert_eq!(
        callback
            .post_raw(serde_json::to_vec(&sample).unwrap())
            .await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(callback.saved_usage(), None);
    let mut latest = usage();
    latest.steps.clear();
    assert_eq!(
        callback.sample(latest.clone()).await,
        StatusCode::NO_CONTENT
    );

    let exit_arrival = at() + Duration::seconds(2);
    callback.set_time(exit_arrival);
    let exit = json!({
        "type": "runner_exited", "runner_id": callback.runner, "exit": { "code": 0 },
        "usage": bad_points,
    });
    assert_eq!(
        callback.post_raw(serde_json::to_vec(&exit).unwrap()).await,
        StatusCode::NO_CONTENT
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
    let callback = Callback::new().await;
    let arrival = at() + Duration::seconds(1);
    callback.set_time(arrival);
    let mut bad_steps = serde_json::to_value(usage()).unwrap();
    bad_steps["steps"][0]["status"] = json!("not a step status");
    let exit = json!({
        "type": "runner_exited", "runner_id": callback.runner, "exit": { "code": 0 },
        "usage": bad_steps,
    });
    assert_eq!(
        callback.post_raw(serde_json::to_vec(&exit).unwrap()).await,
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
