//! Relay authenticated gateway and App events to the hosted runtime.

use std::sync::Arc;
use std::sync::{OnceLock, Weak};

use async_trait::async_trait;
use ciproxy_gateway::boundary::AgentEvents;
use ciproxy_gateway::types::Session;
use ciproxy_github::app::LifecycleOutput;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;

use crate::error::{Error, Result};
use crate::runtime::OwnMachines;

/// Relay authenticated gateway and installation events to the shared runtime.
pub struct Relay {
    /// Installed after the runtime is composed.
    pub(crate) machines: OnceLock<Weak<dyn OwnMachines>>,
}

impl Relay {
    /// Create the gateway relay before the runtime is composed.
    pub fn new() -> Self {
        Self {
            machines: OnceLock::new(),
        }
    }

    /// Bind the completed runtime without a strong ownership cycle.
    pub fn bind(&self, runtime: &Arc<dyn OwnMachines>) -> Result<()> {
        self.machines
            .set(Arc::downgrade(runtime))
            .map_or_else(|_| Err(Error::StateChanged), |_| Ok(()))
    }

    fn runtime(&self) -> Result<Arc<dyn OwnMachines>> {
        self.machines
            .get()
            .and_then(Weak::upgrade)
            .ok_or(Error::StateChanged)
    }
}

impl Default for Relay {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentEvents for Relay {
    fn machine_removed(
        &self,
        workspace_id: uuid::Uuid,
        machine_id: uuid::Uuid,
    ) -> ciproxy_gateway::error::Result<()> {
        match self.runtime() {
            Ok(runtime) => runtime.enqueue_machine_removed(workspace_id, machine_id),
            Err(error) => {
                tracing::warn!(%error, "machine removal relay unavailable; saved state retains recovery")
            }
        }
        Ok(())
    }

    fn receive(
        &self,
        session: Session,
        message: AgentMessage,
    ) -> ciproxy_gateway::error::Result<()> {
        match self.runtime() {
            Ok(runtime) => runtime.enqueue_agent(session, message),
            Err(error) => {
                tracing::warn!(%error, "agent event relay unavailable; full reports retain recovery")
            }
        }
        Ok(())
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
