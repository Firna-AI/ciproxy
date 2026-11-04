//! Fake GitHub HTTP with real SDK wire envelopes and captured acknowledgements.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::store::Store;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use super::resource_http::{ResourceHttp, respond};
use super::support::id;

pub(super) struct Server {
    pub(super) origin: String,
    pub(super) state: Arc<FakeState>,
    task: JoinHandle<()>,
}

pub(super) struct FakeState {
    origin: String,
    store: Option<Arc<dyn Store>>,
    next: AtomicUsize,
    pub(super) acknowledgements: Mutex<Vec<DemandPhase>>,
    pub(super) requests: Mutex<Vec<(Method, String, Value)>>,
    pub(super) tokens: AtomicUsize,
    pub(super) body_override: Mutex<Option<String>>,
    pub(super) reject_repository_token: AtomicBool,
    runner_present: AtomicBool,
    pub(super) resource: ResourceHttp,
    pub(super) fail_later_facts: AtomicBool,
    pub(super) facts_reads: AtomicUsize,
}

impl Server {
    pub(super) async fn start(store: Option<Arc<dyn Store>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(FakeState {
            origin: origin.clone(),
            store,
            next: AtomicUsize::new(0),
            acknowledgements: Mutex::new(vec![]),
            requests: Mutex::new(vec![]),
            tokens: AtomicUsize::new(0),
            body_override: Mutex::new(None),
            reject_repository_token: AtomicBool::new(false),
            runner_present: AtomicBool::new(true),
            resource: ResourceHttp::new(),
            fail_later_facts: AtomicBool::new(false),
            facts_reads: AtomicUsize::new(0),
        });
        let router = Router::new().fallback(handle).with_state(state.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            origin,
            state,
            task,
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn handle(
    State(state): State<Arc<FakeState>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    let path = uri.path();
    state
        .requests
        .lock()
        .unwrap()
        .push((method.clone(), uri.to_string(), body.clone()));
    let app_jwt = headers
        .get("Authorization")
        .is_some_and(|value| value == "Bearer test-app-jwt");
    if let Some(response) = respond(&state.resource, &method, path, &body, app_jwt) {
        return response;
    }
    if path.ends_with("/access_tokens") {
        let number = state.tokens.fetch_add(1, Ordering::SeqCst) + 1;
        let expiry = if number == 1 {
            "2026-11-04T12:10:00Z"
        } else {
            "2026-11-04T12:20:00Z"
        };
        return (
            StatusCode::CREATED,
            Json(json!({"token":format!("test-installation-token-{number}"),"expires_at":expiry})),
        )
            .into_response();
    }
    if path.ends_with("/registration-token") {
        return (
            StatusCode::CREATED,
            Json(json!({"token":"test-registration-token"})),
        )
            .into_response();
    }
    if path.ends_with("/runner-registration") {
        return Json(json!({"url":format!("{}/actions", state.origin),"token":"e30.eyJleHAiOjQxMDI0NDQ4MDB9.test"})).into_response();
    }
    if path.ends_with("/sessions") && method == Method::POST {
        return Json(json!({"sessionId":"session-1","messageQueueUrl":format!("{}/queue", state.origin),"messageQueueAccessToken":"test-queue-token"})).into_response();
    }
    if path.ends_with("/sessions/session-1") && method == Method::DELETE {
        return StatusCode::NO_CONTENT.into_response();
    }
    if path.ends_with("/53/generatejitconfig") {
        return Json(json!({"runner":{"id":71,"name":"runner-71","runnerScaleSetId":53},"encodedJITConfig":"test-one-use-configuration"})).into_response();
    }
    if path.ends_with("/agents/71") && method == Method::DELETE {
        return if state.runner_present.swap(false, Ordering::SeqCst) {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::NOT_FOUND
        }
        .into_response();
    }
    if path.ends_with("/agents/73") && method == Method::DELETE {
        return (
            StatusCode::CONFLICT,
            Json(json!({"typeName":"JobStillRunningException","message":"The runner has a job"})),
        )
            .into_response();
    }
    if path == "/queue" && method == Method::GET {
        let index = state.next.fetch_add(1, Ordering::SeqCst);
        let phase = [
            "JobAssigned",
            "JobAssigned",
            "JobStarted",
            "JobCompleted",
            "JobAssigned",
        ][index];
        let started = if phase == "JobStarted" || phase == "JobCompleted" {
            "2026-11-04T10:46:21.661245605Z"
        } else {
            "0001-01-01T00:00:00Z"
        };
        let finish = if phase == "JobCompleted" {
            "2026-11-04T10:46:24.317465059Z"
        } else {
            "0001-01-01T00:00:00Z"
        };
        let jobs = json!([{"messageType":phase,"jobId":"01a277ed-2171-562e-8431-03d0033029ab","jobDisplayName":"probe",
            "jobWorkflowRef":"shared/reusable/.github/workflows/spike.yml@refs/heads/main","ownerName":"customer","repositoryName":"private",
            "workflowRunId":31,"eventName":"workflow_dispatch","runnerRequestId":0,"queueTime":"0001-01-01T00:00:00Z",
            "scaleSetAssignTime":"2026-11-04T10:46:21.620395766Z","runnerAssignTime":started,"finishTime":finish,"runnerId":15,"result":"succeeded"}]);
        let body = state
            .body_override
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| jobs.to_string());
        return Json(json!({"messageId":7+index,"messageType":"RunnerScaleSetJobMessages","body":body,"statistics":{"totalAssignedJobs":1}})).into_response();
    }
    if path.starts_with("/queue/") && method == Method::DELETE {
        let jobs = state
            .store
            .as_ref()
            .unwrap()
            .github_jobs(id(1), id(5))
            .unwrap();
        state
            .acknowledgements
            .lock()
            .unwrap()
            .extend(jobs.into_iter().map(|job| job.phase));
        return StatusCode::NO_CONTENT.into_response();
    }
    if path.ends_with("/actions/runs/31") {
        if state.facts_reads.fetch_add(1, Ordering::SeqCst) > 0
            && state.fail_later_facts.load(Ordering::SeqCst)
        {
            return StatusCode::NOT_FOUND.into_response();
        }
        return Json(json!({"id":31,"repository":{"id":23,"full_name":"customer/private"},"event":"pull_request_target",
            "head_branch":"untrusted-feature","head_sha":"0123456789abcdef","head_repository":null,
            "path":".github/workflows/caller.yml@main","pull_requests":[]})).into_response();
    }
    if path.ends_with("/installation/repositories") {
        if state.reject_repository_token.swap(false, Ordering::SeqCst) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        return Json(json!({"repositories":[{"id":23,"full_name":"customer/private","private":true,"default_branch":"trunk"},
            {"id":29,"full_name":"customer/public","private":false,"default_branch":"main"}]})).into_response();
    }
    if path.ends_with("/runner-groups") && method == Method::GET {
        return Json(json!({"runner_groups":[]})).into_response();
    }
    if path.ends_with("/runner-groups") && method == Method::POST {
        return (
            StatusCode::CREATED,
            Json(json!({"id":47,"name":"ciproxy-test","inherited":false})),
        )
            .into_response();
    }
    if path.ends_with("/runner-groups/47/repositories") && method == Method::PUT {
        return StatusCode::NO_CONTENT.into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}
