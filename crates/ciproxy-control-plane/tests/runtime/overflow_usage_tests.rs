//! Boat usage through the authenticated HTTP route and real store.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::job_usage::StoredJobUsage;
use uuid::Uuid;

use super::overflow_usage_support::{Callback, usage};
use super::support::at;

#[tokio::test]
async fn boat_samples_use_actual_size_receipt_time_and_sandbox_token() {
    let callback = Callback::new().await;
    let arrival = at() + Duration::seconds(20);
    callback.set_time(arrival);
    assert_eq!(
        callback.sample_for(Uuid::new_v4(), usage()).await,
        StatusCode::UNAUTHORIZED
    );
    let beyond_cpu = JobUsage {
        cpu_milli_vcpu: vec![8001],
        ..usage()
    };
    let beyond_memory = JobUsage {
        peak_memory_mib: vec![15_259],
        ..usage()
    };
    let beyond_time = JobUsage {
        first_point_at_ms: arrival.timestamp_millis() + 299_001,
        ..usage()
    };
    for report in [beyond_cpu, beyond_memory, beyond_time] {
        assert_eq!(
            callback.sample(report).await,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(callback.saved_usage(), None);
    let report = JobUsage {
        first_point_at_ms: arrival.timestamp_millis() + 299_000,
        ..usage()
    };
    assert_eq!(
        callback.sample(report.clone()).await,
        StatusCode::NO_CONTENT
    );
    let mut expected = report;
    expected.steps[0].name = "Pre Run script".into();
    assert_eq!(
        callback.saved_usage(),
        Some(StoredJobUsage {
            usage: expected,
            is_final: false,
            received_at: arrival,
        })
    );
    assert_eq!(callback.delivery().reported_at, None);
}

#[tokio::test]
async fn boat_samples_limit_concurrent_writes_and_replace_after_ten_seconds() {
    let callback = Callback::new().await;
    callback.set_time(at() + Duration::seconds(1));
    let (first, second) = tokio::join!(callback.sample(usage()), callback.sample(usage()));
    let mut statuses = [first, second];
    statuses.sort_unstable();
    assert_eq!(
        statuses,
        [StatusCode::NO_CONTENT, StatusCode::TOO_MANY_REQUESTS]
    );
    let initial = callback.saved_usage().unwrap();
    let newer = JobUsage {
        cpu_milli_vcpu: vec![2100, 2200],
        peak_memory_mib: vec![8000, 8200],
        oom_kills: Some(vec![0, 0]),
        steps: vec![JobStep {
            name: "Build".into(),
            timing: JobStepTiming::Skipped,
        }],
        ..usage()
    };
    callback.set_time(at() + Duration::milliseconds(10_999));
    assert_eq!(
        callback.sample(newer.clone()).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(callback.saved_usage(), Some(initial));
    let arrival = at() + Duration::seconds(11);
    callback.set_time(arrival);
    assert_eq!(callback.sample(newer.clone()).await, StatusCode::NO_CONTENT);
    assert_eq!(
        callback.saved_usage(),
        Some(StoredJobUsage {
            usage: newer,
            is_final: false,
            received_at: arrival,
        })
    );
}

#[tokio::test]
async fn boat_samples_are_refused_after_cleanup_without_an_exit_report() {
    let mut callback = Callback::new().await;
    let fixture = &callback.harness.fixture;
    let ended_at = at() + Duration::seconds(30);
    callback.set_time(ended_at);
    callback.job.phase = DemandPhase::Completed;
    callback.job.runner_github_id = Some(100);
    callback.job.completed_at = Some(ended_at);
    fixture
        .store
        .save_github_job(fixture.workspace, &callback.job, ended_at)
        .unwrap();
    callback
        .harness
        .service
        .cleanup(fixture.workspace)
        .await
        .unwrap();
    assert_eq!(callback.delivery().ends_at, Some(ended_at));
    assert_eq!(callback.delivery().reported_at, None);
    assert_eq!(callback.sample(usage()).await, StatusCode::GONE);
    assert_eq!(callback.saved_usage(), None);
}

#[tokio::test]
async fn boat_saves_final_usage_after_claim_and_refuses_later_samples() {
    let mut callback = Callback::new().await;
    let arrival = at() + Duration::seconds(1);
    callback.set_time(arrival);
    callback.harness.fixture.reject_boat_exit_claims();
    assert_eq!(
        callback.exit(usage()).await,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(callback.saved_usage(), None);
    callback.harness.fixture.allow_boat_exit_claims();
    assert_eq!(callback.sample(usage()).await, StatusCode::NO_CONTENT);
    let exit_arrival = at() + Duration::seconds(2);
    callback.set_time(exit_arrival);
    callback
        .harness
        .ports
        .delete_fail
        .store(true, Ordering::SeqCst);
    let final_usage = JobUsage {
        cpu_milli_vcpu: vec![2500],
        steps: vec![JobStep {
            name: "Post Run make clean".into(),
            timing: JobStepTiming::Skipped,
        }],
        ..usage()
    };
    assert_eq!(
        callback.exit(final_usage.clone()).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(callback.delivery().reported_at, Some(exit_arrival));
    assert_eq!(callback.delivery().ends_at, None);
    let mut expected = final_usage;
    expected.steps[0].name = "Post Run script".into();
    let final_series = StoredJobUsage {
        usage: expected,
        is_final: true,
        received_at: exit_arrival,
    };
    assert_eq!(callback.saved_usage(), Some(final_series.clone()));
    callback.set_time(at() + Duration::seconds(30));
    assert_eq!(callback.sample(usage()).await, StatusCode::GONE);
    assert_eq!(callback.saved_usage(), Some(final_series));
    assert_eq!(callback.exit(usage()).await, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn boat_invalid_final_points_do_not_change_the_exit_or_latest_samples() {
    let callback = Callback::new().await;
    callback.set_time(at() + Duration::seconds(1));
    assert_eq!(callback.sample(usage()).await, StatusCode::NO_CONTENT);
    let latest = at() + Duration::seconds(11);
    callback.set_time(latest);
    let newest = JobUsage {
        cpu_milli_vcpu: vec![3500],
        steps: vec![],
        ..usage()
    };
    assert_eq!(
        callback.sample(newest.clone()).await,
        StatusCode::NO_CONTENT
    );
    let exit_arrival = at() + Duration::seconds(12);
    callback.set_time(exit_arrival);
    let invalid = JobUsage {
        cpu_milli_vcpu: vec![8001],
        ..usage()
    };
    assert_eq!(callback.exit(invalid).await, StatusCode::NO_CONTENT);
    assert_eq!(callback.delivery().ends_at, Some(exit_arrival));
    assert_eq!(callback.harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(
        callback.saved_usage(),
        Some(StoredJobUsage {
            usage: newest,
            is_final: false,
            received_at: latest,
        })
    );
}

#[tokio::test]
async fn boat_final_usage_store_failure_does_not_change_the_exit() {
    let mut callback = Callback::new().await;
    callback.harness.fixture.reject_final_usage();
    let exit_arrival = at() + Duration::seconds(1);
    callback.set_time(exit_arrival);
    assert_eq!(callback.exit(usage()).await, StatusCode::NO_CONTENT);
    assert_eq!(callback.delivery().reported_at, Some(exit_arrival));
    assert_eq!(callback.delivery().ends_at, Some(exit_arrival));
    assert_eq!(callback.harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(callback.saved_usage(), None);
}
