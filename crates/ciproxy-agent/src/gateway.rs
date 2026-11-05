//! Reconnect with a growing delay while preserving all VM ownership.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::version::ProtocolVersion;

use crate::boundaries::{Connection, Delay};
use crate::connection_types::ControlEvent;
use crate::controller::Agent;
use crate::error::{Error, ErrorResultExt, Result};
use crate::state::Identity;

#[cfg_attr(test, unimock::unimock(api = GatewayMock))]
#[async_trait]
pub(crate) trait Gateway: Send + Sync {
    async fn reconnect(&self, identity: &Identity) -> Result<()>;
}

pub(crate) struct ReconnectingGateway {
    pub(crate) connection: Arc<dyn Connection>,
    pub(crate) agent: Arc<dyn Agent>,
    pub(crate) delay: Arc<dyn Delay>,
}

enum Authentication {
    Accepted,
    Rejected,
}

#[async_trait]
impl Gateway for ReconnectingGateway {
    async fn reconnect(&self, identity: &Identity) -> Result<()> {
        let mut seconds = 1;
        loop {
            match self.authenticate(identity).await {
                Ok(Authentication::Accepted) => return Ok(()),
                Ok(Authentication::Rejected) => {
                    self.agent.remove().await?;
                    return Err(Error::Revoked);
                }
                Err(error) => {
                    tracing::warn!(error = %error, seconds, "control plane unavailable; preserve VMs and reconnect");
                    self.delay.wait(seconds).await;
                    seconds = (seconds * 2).min(60);
                }
            }
        }
    }
}

impl ReconnectingGateway {
    async fn authenticate(&self, identity: &Identity) -> Result<Authentication> {
        self.connection.connect().await.context("connect gateway")?;
        self.connection
            .send(&AgentMessage::Hello {
                protocol_version: ProtocolVersion::CURRENT,
                workspace_id: identity.workspace_id.clone(),
                machine_id: identity.machine_id.clone(),
                credential: identity.credential.clone(),
                report: self.agent.report().await?,
            })
            .await
            .context("send machine hello")?;
        match self
            .connection
            .receive_handshake()
            .await
            .context("receive gateway handshake")?
        {
            ControlEvent::Message(message) => match *message {
                ControlMessage::Welcome { protocol_version }
                    if protocol_version == ProtocolVersion::CURRENT =>
                {
                    Ok(Authentication::Accepted)
                }
                message @ ControlMessage::Remove { .. } => {
                    self.agent.control(message).await?;
                    Ok(Authentication::Rejected)
                }
                _ => Err(Error::ProtocolVersion),
            },
            ControlEvent::CredentialRejected => Ok(Authentication::Rejected),
        }
    }
}

#[cfg(test)]
#[path = "_tests_/gateway_tests.rs"]
mod gateway_tests;

#[cfg(test)]
#[path = "_tests_/gateway_transport_tests.rs"]
mod gateway_transport_tests;
