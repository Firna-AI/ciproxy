//! Relay authenticated gateway and App events to the hosted runtime.

use std::sync::Arc;
use std::sync::{OnceLock, Weak};

use async_trait::async_trait;
use ciproxy_gateway::boundary::AgentEvents;
use ciproxy_gateway::types::Session;
use ciproxy_github::app::LifecycleOutput;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use tokio::runtime::Handle;

use crate::error::{Error, Result};
use crate::runtime::OwnMachines;

/// Relay authenticated gateway and installation events to the shared runtime.
pub(crate) struct Relay {
    /// Installed after the runtime is composed.
    pub(crate) machines: OnceLock<Weak<dyn OwnMachines>>,
    /// Runtime handle used by the synchronous gateway event boundary.
    pub(crate) handle: Handle,
}

impl Relay {
    fn runtime(&self) -> Result<Arc<dyn OwnMachines>> {
        self.machines
            .get()
            .and_then(Weak::upgrade)
            .ok_or(Error::StateChanged)
    }
}

impl AgentEvents for Relay {
    fn machine_removed(
        &self,
        workspace_id: uuid::Uuid,
        machine_id: uuid::Uuid,
    ) -> ciproxy_gateway::error::Result<()> {
        let runtime = match self.runtime() {
            Ok(runtime) => runtime,
            Err(_) => return Err(ciproxy_gateway::error::Error::UnexpectedMessage),
        };
        self.handle
            .block_on(runtime.machine_removed(workspace_id, machine_id))
            .map_or_else(
                |_| Err(ciproxy_gateway::error::Error::UnexpectedMessage),
                |_| Ok(()),
            )
    }
    fn receive(
        &self,
        session: Session,
        message: AgentMessage,
    ) -> ciproxy_gateway::error::Result<()> {
        let result = self.runtime();
        match result {
            Ok(runtime) => self
                .handle
                .block_on(runtime.agent(session.workspace_id, session.machine_id, message))
                .map_or_else(
                    |_| Err(ciproxy_gateway::error::Error::UnexpectedMessage),
                    |_| Ok(()),
                ),
            Err(_) => Err(ciproxy_gateway::error::Error::UnexpectedMessage),
        }
    }
}

#[async_trait]
impl LifecycleOutput for Relay {
    async fn changed(&self, event: LifecycleEvent) -> ciproxy_github::error::Result<()> {
        let runtime = match self.runtime() {
            Ok(runtime) => runtime,
            Err(_) => return Err(ciproxy_github::error::Error::OrganizationInactive),
        };
        match runtime.lifecycle(event).await {
            Ok(()) => Ok(()),
            Err(_) => Err(ciproxy_github::error::Error::OrganizationInactive),
        }
    }
}
