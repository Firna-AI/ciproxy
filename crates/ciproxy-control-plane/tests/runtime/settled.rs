//! Explicit completion boundaries for tests that inspect dispatched remote effects.

use async_trait::async_trait;
use ciproxy_control_plane::error::{Error, Result};
use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::github_lifecycle::{LifecycleEvent, RemovalCause};
use uuid::Uuid;

#[async_trait]
pub(super) trait Settled {
    async fn settled_tick(&self) -> Result<()>;
    async fn settled_job(&self, workspace: Uuid, job: GithubJob) -> Result<()>;
    async fn settled_agent(
        &self,
        workspace: Uuid,
        machine: Uuid,
        message: AgentMessage,
    ) -> Result<()>;
    async fn settled_lifecycle(&self, event: LifecycleEvent) -> Result<()>;
}

#[async_trait]
impl Settled for MachineRuntime {
    async fn settled_tick(&self) -> Result<()> {
        self.tick().await?;
        match self.quiesce().await {
            Err(error @ Error::TaskJoin(_)) => Err(error),
            Ok(()) | Err(_) => Ok(()),
        }
    }

    async fn settled_job(&self, workspace: Uuid, job: GithubJob) -> Result<()> {
        self.job(workspace, job).await?;
        self.quiesce().await
    }

    async fn settled_agent(
        &self,
        workspace: Uuid,
        machine: Uuid,
        message: AgentMessage,
    ) -> Result<()> {
        self.agent(workspace, machine, message).await?;
        self.quiesce().await
    }

    async fn settled_lifecycle(&self, event: LifecycleEvent) -> Result<()> {
        let result = self.lifecycle(event.clone()).await;
        if matches!(
            event,
            LifecycleEvent::Removed {
                cause: RemovalCause::WorkspaceTakeout,
                ..
            }
        ) && matches!(result, Ok(()) | Err(Error::StateChanged))
        {
            return self.quiesce_lifecycle(&event).await;
        }
        self.quiesce().await?;
        result
    }
}
