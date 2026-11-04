//! Registration happens once. Reinstall uses the existing root-only identity.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::WorkspaceId;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;

use crate::boundaries::{Connection, Host, Service, Store, WorkspaceResolver};
use crate::controller::Agent;
use crate::error::{Error, Result};
use crate::state::{Identity, Mode};

#[async_trait]
pub(crate) trait Install: Send + Sync {
    async fn install(&self, token: Secret) -> Result<()>;
}

pub(crate) struct Installer {
    pub(crate) store: Arc<dyn Store>,
    pub(crate) host: Arc<dyn Host>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) resolver: Arc<dyn WorkspaceResolver>,
    pub(crate) connection: Arc<dyn Connection>,
    pub(crate) agent: Arc<dyn Agent>,
}

#[async_trait]
impl Install for Installer {
    async fn install(&self, token: Secret) -> Result<()> {
        self.host.require_root()?;
        let mut state = self.store.load()?;
        if state.mode != Mode::Removed && state.identity.is_some() {
            return self.service.install();
        }
        if !state.runners.is_empty() {
            return Err(Error::Revoked);
        }
        let workspace_id = self.resolver.workspace(&token).await?;
        self.store.save(&state)?;
        let report = self.agent.report().await?;
        self.connection.connect().await?;
        self.connection
            .send(&AgentMessage::Register {
                protocol_version: ProtocolVersion::CURRENT,
                workspace_id: workspace_id.clone(),
                install_token: token,
                report,
            })
            .await?;
        let ControlMessage::Registered {
            workspace_id: registered_workspace,
            machine_id,
            credential,
        } = self.connection.receive_handshake().await?
        else {
            return Err(Error::IdentityMismatch);
        };
        if registered_workspace != workspace_id {
            return Err(Error::IdentityMismatch);
        }
        state.identity = Some(Identity {
            workspace_id,
            machine_id,
            credential,
        });
        state.mode = Mode::Active;
        self.store.save(&state)?;
        self.service.install()
    }
}

pub(crate) struct PendingWorkspaceResolver;

#[async_trait]
impl WorkspaceResolver for PendingWorkspaceResolver {
    async fn workspace(&self, _token: &Secret) -> Result<WorkspaceId> {
        Err(Error::WorkspaceResolutionUnavailable)
    }
}

#[cfg(test)]
#[path = "_tests_/installation_tests.rs"]
mod installation_tests;
