//! Controlled in-flight poll and literal job facts for retired-label scenarios.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_github::error::{Error, Result};
use ciproxy_github::listener::JobOutput;
use ciproxy_github::scale_sets::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::scale_sets::ScaleSet;
use scaleset::{JobRequestId, MessageId, ScaleSetMessage};
use tokio::sync::Notify;
use uuid::Uuid;

#[derive(Clone, Default)]
pub(super) struct Queue {
    pub(super) entered: Arc<Notify>,
    pub(super) release: Arc<Notify>,
    pub(super) capacities: Arc<Mutex<Vec<u32>>>,
    pub(super) acquired: Arc<Mutex<Vec<i64>>>,
    pub(super) acknowledged: Arc<Mutex<Vec<i64>>>,
    pub(super) closed: Arc<AtomicBool>,
    pub(super) empty: Arc<AtomicBool>,
}

#[async_trait]
impl ScaleSetFactory for Queue {
    async fn for_organization(&self, _: &Organization) -> Result<Arc<dyn ScaleSetApi>> {
        Ok(Arc::new(self.clone()))
    }
}

#[async_trait]
impl ScaleSetApi for Queue {
    async fn ensure(&self, _: &str, _: i64, _: Option<i64>) -> Result<i64> {
        Err(Error::AccessUnavailable(501))
    }
    async fn open(&self, _: i64, _: &str) -> Result<Box<dyn MessageQueue>> {
        Ok(Box::new(self.clone()))
    }
    async fn register(&self, _: i64, _: &str) -> Result<JitRegistration> {
        Err(Error::AccessUnavailable(501))
    }
    async fn remove_runner(&self, _: i64) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
    async fn remove_scale_set(&self, _: i64) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
    async fn rename_scale_set(&self, _: i64, _: &str) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
}

#[async_trait]
impl MessageQueue for Queue {
    async fn poll(&self, _: Option<MessageId>, capacity: u32) -> Result<Option<ScaleSetMessage>> {
        let first = {
            let mut capacities = self.capacities.lock().unwrap();
            capacities.push(capacity);
            capacities.len() == 1
        };
        if first {
            self.entered.notify_one();
            self.release.notified().await;
        }
        if self.empty.load(Ordering::SeqCst) {
            return Ok(None);
        }
        let mut message = ScaleSetMessage::default();
        message.id = MessageId(7);
        message.jobs = serde_json::from_str(r#"[
            {"messageType":"JobCompleted","jobId":"00000000-0000-0000-0000-000000000071","runnerRequestId":81,"workflowRunId":31,"ownerName":"customer","repositoryName":"private","jobDisplayName":"test","jobWorkflowRef":"shared/reuse.yml@main","scaleSetAssignTime":"2026-11-06T11:00:00Z","runnerAssignTime":"2026-11-06T11:00:01Z","finishTime":"2026-11-06T12:00:01Z","runnerId":15,"result":"succeeded"},
            {"messageType":"JobAvailable","jobId":"00000000-0000-0000-0000-000000000072","runnerRequestId":82,"workflowRunId":31,"ownerName":"customer","repositoryName":"private","jobDisplayName":"waiting","jobWorkflowRef":"shared/reuse.yml@main"}
        ]"#).unwrap();
        Ok(Some(message))
    }
    async fn acquire(&self, requests: &[JobRequestId]) -> Result<()> {
        self.acquired
            .lock()
            .unwrap()
            .extend(requests.iter().map(|request| request.get()));
        Ok(())
    }
    async fn acknowledge(&self, id: MessageId) -> Result<()> {
        self.acknowledged.lock().unwrap().push(id.0);
        Ok(())
    }
    async fn close(&self) -> Result<()> {
        self.closed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[async_trait]
impl JobOutput for Queue {
    async fn changed(&self, _: GithubJob) -> Result<()> {
        Ok(())
    }
}

pub(super) fn started(set: &ScaleSet) -> GithubJob {
    GithubJob {
        id: Uuid::from_u128(0x71),
        organization_id: set.organization_id,
        scale_set_id: set.id,
        label_id: set.label_id,
        actions_job_id: None,
        request_id: 81,
        run_id: 31,
        repository: "customer/private".to_owned(),
        name: "test".to_owned(),
        workflow_ref: "shared/reuse.yml@main".to_owned(),
        phase: DemandPhase::Started,
        result: None,
        assigned_at: Some("2026-11-06T11:00:00Z".parse().unwrap()),
        started_at: Some("2026-11-06T11:00:01Z".parse().unwrap()),
        completed_at: None,
        runner_github_id: Some(15),
        facts: RunFacts {
            id: 31,
            repository: RepositoryIdentity {
                id: 23,
                full_name: "customer/private".to_owned(),
            },
            event: "push".to_owned(),
            head_branch: Some("main".to_owned()),
            head_sha: "0123456789abcdef".to_owned(),
            head_repository: None,
        },
    }
}
