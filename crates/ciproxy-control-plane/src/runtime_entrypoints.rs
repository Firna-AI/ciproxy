//! Short serialized dispatch sections for runtime entry points.

use async_trait::async_trait;
use ciproxy_gateway::types::Session;
use ciproxy_github::persistence::lease::LeaseStore;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::github_lifecycle::{LifecycleEvent, RemovalCause};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::{MachineRuntime, OwnMachines};
use crate::runtime_agent_event::AgentEvent;
use crate::runtime_payloads::WorkerJob;

#[async_trait]
impl OwnMachines for MachineRuntime {
    fn enqueue_agent(&self, session: Session, message: AgentMessage) {
        if let Ok(tasks) = self.task_scope() {
            tasks.enqueue_agent(self.worker(), session, message);
        }
    }

    fn enqueue_machine_removed(&self, workspace: Uuid, machine: Uuid) {
        if let Ok(tasks) = self.task_scope() {
            tasks.enqueue_removed(self.worker(), workspace, machine);
        }
    }

    fn cancel_tasks(&self) {
        if let Ok(tasks) = self.task_scope() {
            tasks.cancel();
        }
    }

    async fn shutdown(&self) -> Result<()> {
        self.task_scope()?.shutdown().await
    }

    async fn machine_removed(&self, workspace_id: Uuid, machine_id: Uuid) -> Result<()> {
        let _operation = self.operation.lock().await;
        let Some(lease) = self.store.own_runner_lease(workspace_id).await.internal()? else {
            return Err(Error::StateChanged);
        };
        lease.check().await.internal()?;
        let input = self.load(workspace_id).await?;
        for runner in &input.runners {
            if matches!(runner.runner.place,RunnerPlace::Machine{machine_id:owner,..}if owner==machine_id)
                && runner.runner.state != RunnerState::Stopped
            {
                self.stop(&input, runner).await?;
            }
        }
        Ok(())
    }
    async fn tick(&self) -> Result<()> {
        let store = self.store.clone();
        let workspaces = tokio::task::spawn_blocking(move || store.routing_workspaces())
            .await
            .internal()?
            .internal()?;
        for workspace_id in workspaces {
            let _operation = self.operation.lock().await;
            if let Err(error) = self.reconcile_workspace(workspace_id).await {
                tracing::warn!(%workspace_id, %error, "own-machine reconciliation will retry");
            }
        }
        Ok(())
    }

    async fn job(&self, workspace_id: Uuid, job: GithubJob) -> Result<()> {
        let _operation = self.operation.lock().await;
        let Some(lease) = self.store.own_runner_lease(workspace_id).await.internal()? else {
            return Err(Error::StateChanged);
        };
        lease.check().await.internal()?;
        let input = self.load(workspace_id).await?;
        self.route_boat_job(&input, &job).await?;
        self.route_job(&input, &job).await
    }

    async fn agent(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        message: AgentMessage,
    ) -> Result<()> {
        let _operation = self.operation.lock().await;
        let Some((message, worker)) = AgentEvent::from_message(message) else {
            return Ok(());
        };
        self.agent_event(
            workspace_id,
            machine_id,
            message,
            worker.map(WorkerJob::direct),
        )
        .await
    }

    async fn lifecycle(&self, event: LifecycleEvent) -> Result<()> {
        let workspace_id = event.organization().workspace_id;
        let _operation = self.operation.lock().await;
        match event {
            LifecycleEvent::Inactive { organization, .. } => self.inactive(&organization).await,
            LifecycleEvent::Removed {
                organization,
                cause,
                ..
            } => match cause {
                RemovalCause::Owner => self.owner_removed(&organization).await,
                RemovalCause::WorkspaceTakeout => self.takeout(&organization).await,
            },
            LifecycleEvent::Active(_) | LifecycleEvent::Suspended(_) => {
                self.reconcile_workspace(workspace_id).await
            }
        }
    }
}
