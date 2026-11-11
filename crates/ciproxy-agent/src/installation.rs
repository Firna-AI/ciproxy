//! Registration happens once. Reinstall uses the existing root-only identity.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;

use crate::boundaries::{Connection, Host, Service, Store};
use crate::connection_types::ControlEvent;
use crate::controller::Agent;
use crate::endpoint::{AgentEndpoint, EndpointSettings};
use crate::error::{Error, ErrorResultExt, Result};
use crate::host_preparation::Preparation;
use crate::state::{Identity, Mode};

#[async_trait]
pub(crate) trait Install: Send + Sync {
    async fn install(&self, token: Secret) -> Result<()>;
}

pub(crate) struct Installer {
    pub(crate) endpoint: AgentEndpoint,
    pub(crate) endpoints: Arc<dyn EndpointSettings>,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) host: Arc<dyn Host>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) connection: Arc<dyn Connection>,
    pub(crate) agent: Arc<dyn Agent>,
    pub(crate) preparation: Arc<dyn Preparation>,
}

#[async_trait]
impl Install for Installer {
    async fn install(&self, token: Secret) -> Result<()> {
        self.host.require_root()?;
        self.preparation.prepare().await?;
        let mut state = self.store.load()?;
        if state.mode != Mode::Removed && state.identity.is_some() {
            self.endpoints.save(&self.endpoint)?;
            return self.service.install();
        }
        if !state.runners.is_empty() {
            return Err(Error::Revoked);
        }
        self.store.save(&state)?;
        let report = self.agent.report().await?;
        self.connection
            .connect()
            .await
            .context("connect registration")?;
        self.connection
            .send(&AgentMessage::Register {
                protocol_version: ProtocolVersion::CURRENT,
                install_token: token,
                report,
            })
            .await
            .context("send registration")?;
        let ControlEvent::Message(message) = self
            .connection
            .receive_handshake()
            .await
            .context("receive registration")?
        else {
            return Err(Error::IdentityMismatch);
        };
        let ControlMessage::Registered {
            workspace_id,
            machine_id,
            credential,
        } = *message
        else {
            return Err(Error::IdentityMismatch);
        };
        state.identity = Some(Identity {
            workspace_id,
            machine_id,
            credential,
        });
        state.mode = Mode::Active;
        self.store.save(&state)?;
        self.endpoints.save(&self.endpoint)?;
        self.service.install()
    }
}

#[cfg(test)]
#[path = "_tests_/installation_tests.rs"]
mod installation_tests;

#[cfg(test)]
#[path = "_tests_/endpoint_install_tests.rs"]
mod endpoint_install_tests;
