//! Real sandbox callback fixtures with controlled arrival times.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{DateTime, Utc};
use ciproxy_control_plane::boat_callback;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::job_usage::StoredJobUsage;
use ciproxy_store_interface::overflow::record::StartRecord;
use tower::ServiceExt;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::settled::Settled;
use super::support::at;

pub(super) struct Callback {
    pub(super) harness: Harness,
    pub(super) job: GithubJob,
    token: String,
    runner: Uuid,
    router: Router,
}

impl Callback {
    pub(super) async fn new() -> Self {
        let harness = Harness::new(false, false).await;
        let job = harness.queued();
        harness.runtime.settled_tick().await.unwrap();
        let (token, runner) = {
            let inputs = harness.ports.inputs.lock().unwrap();
            let fields = &inputs[0];
            (
                fields["exit_report_token"].as_str().unwrap().to_owned(),
                fields["configure"]["runner_id"]
                    .as_str()
                    .unwrap()
                    .parse()
                    .unwrap(),
            )
        };
        let router = boat_callback::router(harness.service.clone());
        Self {
            harness,
            job,
            token,
            runner,
            router,
        }
    }

    pub(super) fn set_time(&self, now: DateTime<Utc>) {
        *self.harness.ports.now.lock().unwrap() = now;
    }

    pub(super) fn saved_usage(&self) -> Option<StoredJobUsage> {
        let fixture = &self.harness.fixture;
        fixture
            .store
            .job_usage(fixture.workspace, self.runner)
            .unwrap()
    }

    pub(super) fn delivery(&self) -> StartRecord {
        let fixture = &self.harness.fixture;
        fixture
            .store
            .boat_runtime_snapshot(fixture.workspace)
            .unwrap()
            .starts
            .remove(0)
    }

    pub(super) async fn sample_for(&self, runner: Uuid, usage: JobUsage) -> StatusCode {
        self.post(GuestToAgent::UsageSamples {
            runner_id: RunnerId(runner.to_string()),
            usage,
        })
        .await
    }

    pub(super) async fn sample(&self, usage: JobUsage) -> StatusCode {
        self.sample_for(self.runner, usage).await
    }

    pub(super) async fn exit(&self, usage: JobUsage) -> StatusCode {
        self.post(GuestToAgent::RunnerExited {
            runner_id: RunnerId(self.runner.to_string()),
            exit: RunnerExit { code: Some(0) },
            usage: Some(usage),
        })
        .await
    }

    async fn post(&self, message: GuestToAgent) -> StatusCode {
        let request = Request::post("/hooks/boat/runner-exit")
            .header("authorization", format!("Bearer {}", self.token))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&message).unwrap()))
            .unwrap();
        self.router.clone().oneshot(request).await.unwrap().status()
    }
}

pub(super) fn usage() -> JobUsage {
    JobUsage {
        platform: Platform::LinuxX64,
        first_point_at_ms: at().timestamp_millis(),
        point_seconds: 1,
        cpu_milli_vcpu: vec![7000],
        peak_memory_mib: vec![12_000],
        oom_kills: Some(vec![0]),
        steps: vec![JobStep {
            name: "Pre Run make prepare".into(),
            timing: JobStepTiming::Skipped,
        }],
    }
}
