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
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::job_usage::StoredJobUsage;
use ciproxy_store_interface::overflow::record::StartRecord;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::settled::Settled;
use super::support::at;

pub(super) struct Callback {
    pub(super) harness: Harness,
    pub(super) job: GithubJob,
    pub(super) runner: Uuid,
    token: Secret,
    router: Router,
}

pub(super) struct Sandbox {
    pub(super) runner: Uuid,
    token: Secret,
}

impl Sandbox {
    fn from_registration(fields: &Value) -> Self {
        Self {
            runner: fields["configure"]["runner_id"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
            token: Secret::new(fields["exit_report_token"].as_str().unwrap()),
        }
    }
}

impl Callback {
    pub(super) async fn new() -> Self {
        Self::start(false).await
    }

    pub(super) async fn with_two_deliveries() -> (Self, Sandbox) {
        let callback = Self::start(true).await;
        let second = Sandbox::from_registration(&callback.harness.ports.inputs.lock().unwrap()[1]);
        (callback, second)
    }

    async fn start(with_second_job: bool) -> Self {
        let harness = Harness::new(false, false).await;
        let job = harness.queued();
        if with_second_job {
            harness.queued();
        }
        harness.runtime.settled_tick().await.unwrap();
        let Sandbox { runner, token } =
            Sandbox::from_registration(&harness.ports.inputs.lock().unwrap()[0]);
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
        self.saved_usage_for(self.runner)
    }

    pub(super) fn saved_usage_for(&self, runner: Uuid) -> Option<StoredJobUsage> {
        let fixture = &self.harness.fixture;
        fixture.store.job_usage(fixture.workspace, runner).unwrap()
    }

    pub(super) fn delivery(&self) -> StartRecord {
        let fixture = &self.harness.fixture;
        fixture
            .store
            .boat_runtime_snapshot(fixture.workspace)
            .unwrap()
            .starts
            .into_iter()
            .find(|delivery| delivery.runner_id == self.runner)
            .unwrap()
    }

    pub(super) async fn sample_for(&self, runner: Uuid, usage: JobUsage) -> StatusCode {
        self.post(sample_report(runner, usage)).await
    }

    pub(super) async fn sample_from(&self, sender: &Sandbox, usage: JobUsage) -> StatusCode {
        let body = serde_json::to_vec(&sample_report(self.runner, usage)).unwrap();
        self.post_raw_with_token(body, &sender.token).await
    }

    pub(super) async fn sample(&self, usage: JobUsage) -> StatusCode {
        self.sample_for(self.runner, usage).await
    }

    pub(super) async fn exit(&self, usage: JobUsage) -> StatusCode {
        self.post(GuestToAgent::RunnerExited {
            runner_id: RunnerId(self.runner.to_string()),
            exit: RunnerExit { code: Some(0) },
            usage: Some(usage.into()),
        })
        .await
    }

    async fn post(&self, message: GuestToAgent) -> StatusCode {
        self.post_raw(serde_json::to_vec(&message).unwrap()).await
    }

    pub(super) async fn post_raw(&self, body: Vec<u8>) -> StatusCode {
        self.post_raw_with_token(body, &self.token).await
    }

    async fn post_raw_with_token(&self, body: Vec<u8>, token: &Secret) -> StatusCode {
        let request = Request::post("/hooks/boat/runner-exit")
            .header("authorization", format!("Bearer {}", token.expose_secret()))
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();
        self.router.clone().oneshot(request).await.unwrap().status()
    }
}

fn sample_report(runner: Uuid, usage: JobUsage) -> GuestToAgent {
    GuestToAgent::UsageSamples {
        runner_id: RunnerId(runner.to_string()),
        usage: usage.into(),
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
