//! Slow accounting cannot hold any runner operation or removal.

use std::future::pending;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_control_plane::accounting::{Accounting, AccountingWorker};
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_github::timing::JobTiming;
use ciproxy_metering::service::StoredMetering;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_store_interface::runners::RunnerState;
use tokio::sync::{Notify, watch};
use unimock::Unimock;
use uuid::Uuid;

use super::recovery_support::{gateway, job, other_organization, runtime, saved};
use super::support::{Fixture, at};

struct PendingTiming(Arc<Notify>);

#[async_trait]
impl JobTiming for PendingTiming {
    async fn repair(&self, _: Uuid) -> ciproxy_github::error::Result<()> {
        self.0.notify_one();
        pending().await
    }
}

#[tokio::test]
async fn pending_timing_does_not_block_tick_job_agent_or_removal() {
    let fixture = Fixture::new();
    other_organization(&fixture);
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Busy,
            Some(47),
        )
        .unwrap();
    let entered = Arc::new(Notify::new());
    let runtime = runtime(
        &fixture,
        Arc::new(Unimock::new(())),
        gateway(Arc::new(Mutex::new(Vec::new()))),
    );
    let worker = Arc::new(AccountingWorker::new(
        fixture.store.clone(),
        Arc::new(PendingTiming(entered.clone())),
        Arc::new(StoredMetering::new(fixture.store.clone())),
        Arc::new(Unimock::new(())),
    ));
    let (stop, receiver) = watch::channel(false);
    let running = worker.clone();
    let task = tokio::spawn(async move { running.run(receiver).await });
    worker.request();
    let outcome = tokio::time::timeout(Duration::from_secs(10), async {
        entered.notified().await;
        runtime.tick().await.unwrap();
        runtime.job(fixture.workspace, job(&fixture)).await.unwrap();
        runtime
            .agent(
                fixture.workspace,
                fixture.machine,
                AgentMessage::JobStarted {
                    runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                    job_id: ciproxy_protocol::identifiers::JobId("job".to_owned()),
                },
            )
            .await
            .unwrap();
        let removed = fixture
            .store
            .remove_github_installation(fixture.workspace, fixture.organization, 43, at())
            .unwrap();
        let event = fixture
            .store
            .pending_github_lifecycle(removed.github_id)
            .unwrap()
            .remove(0)
            .event;
        runtime.lifecycle(event).await.unwrap();
        stop.send_replace(true);
        task.await.unwrap();
    })
    .await;
    assert_eq!(
        outcome,
        Ok(()),
        "runner operations stayed blocked while timing was pending"
    );
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
}
