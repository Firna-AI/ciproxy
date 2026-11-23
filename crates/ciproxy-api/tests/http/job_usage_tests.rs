//! Usage in the authorized job detail route, not the job lists.

use axum::http::StatusCode;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::job_usage::StoredJobUsage;
use ciproxy_store_interface::jobs::{Job, JobState};
use serde_json::{Value, json};
use uuid::Uuid;

use super::fixture::Browser;
use super::fixture_io::json;
use super::m18_jobs_support::{actions, runner};
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};

#[tokio::test]
async fn job_detail_shows_saved_usage_only_after_the_exact_job_finishes() {
    let stripe = Stripe::new();
    let pages = Pages::new(stripe.provider());
    stripe.bind(pages.fixture.workspace);
    let browser = pages.fixture.login(7, Some(Role::Admin)).await;
    let store = &pages.fixture.database.store;
    let workspace = pages.fixture.workspace;
    let source = runner(&pages, None, 1001);
    let other = runner(&pages, None, 1002);
    let completed = actions(&pages, &source, 11, "2026-11-06T11:59:00Z");
    let unavailable = actions(&pages, &other, 12, "2026-11-06T11:59:00Z");
    let running = Job {
        state: JobState::Running,
        result: None,
        completed_at: None,
        ..completed.clone()
    };
    store.update_job(workspace, &running).unwrap();
    let first_point_at_ms = at("2026-11-06T11:59:00Z").timestamp_millis();
    assert!(
        store
            .save_job_usage(
                workspace,
                source.id,
                &StoredJobUsage {
                    usage: JobUsage {
                        platform: Platform::LinuxX64,
                        first_point_at_ms,
                        point_seconds: 1,
                        cpu_milli_vcpu: vec![1250],
                        peak_memory_mib: vec![512],
                        oom_kills: Some(vec![1]),
                        steps: vec![JobStep {
                            name: "Build".into(),
                            timing: JobStepTiming::Ran {
                                started_at_ms: Some(first_point_at_ms),
                                ended_at_ms: Some(first_point_at_ms + 1000),
                            },
                        }],
                    },
                    is_final: false,
                    received_at: at("2026-11-06T11:59:01Z"),
                }
            )
            .unwrap()
    );

    let while_running = detail(&pages, &browser, running.id).await;
    store.update_job(workspace, &completed).unwrap();
    let measured = detail(&pages, &browser, completed.id).await;
    let without_report = detail(&pages, &browser, unavailable.id).await;
    assert_eq!(
        json!([
            while_running["usage"],
            measured["usage"],
            without_report["usage"]
        ]),
        json!([
            {"state": "running"},
            {"state": "measured",
             "job": {"average_cpu_milli_vcpu": 1250, "peak_memory_bytes": "536870912", "oom_kills": 1},
             "vm": {"vcpus": 2, "memory_bytes": "8000000000"}, "label_mark": null,
             "chart": {"started_at": "2026-11-06T11:59:00Z", "ended_at": "2026-11-06T11:59:01Z",
                       "point_seconds": 1, "points": [{"average_cpu_milli_vcpu": 1250,
                           "peak_memory_bytes": "536870912", "oom_kills": 1}]},
             "steps": [
                 {"number": 1, "name": "Set up job", "script": false, "skipped": false,
                  "started_at": "2026-11-06T11:59:00Z", "ended_at": "2026-11-06T11:59:00Z",
                  "duration_seconds": "0", "figures": null},
                 {"number": 2, "name": "Build", "script": false, "skipped": false,
                  "started_at": "2026-11-06T11:59:00Z", "ended_at": "2026-11-06T11:59:01Z",
                  "duration_seconds": "1", "figures": {"average_cpu_milli_vcpu": 1250,
                      "peak_memory_bytes": "536870912", "oom_kills": 1}},
                 {"number": 3, "name": "Complete job", "script": false, "skipped": false,
                  "started_at": "2026-11-06T11:59:01Z", "ended_at": "2026-11-06T11:59:01Z",
                  "duration_seconds": "0", "figures": null}
             ]},
            {"state": "unavailable"}
        ])
    );

    let response = pages
        .request(Some(&browser), "GET", "/jobs/history", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let history = json(response).await;
    let jobs = history["jobs"].as_array().unwrap();
    assert_eq!(jobs.len(), 2);
    assert!(jobs.iter().all(|job| job.get("usage").is_none()));
}

async fn detail(pages: &Pages, browser: &Browser, id: Uuid) -> Value {
    let response = pages
        .request(Some(browser), "GET", &format!("/jobs/{id}"), None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    json(response).await
}
