//! Reconnect with a growing delay while preserving all VM ownership.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::version::ProtocolVersion;

use crate::boundaries::{Connection, Delay};
use crate::controller::Agent;
use crate::error::{Error, Result};
use crate::state::Identity;

#[async_trait]
pub(crate) trait Gateway: Send + Sync {
    async fn reconnect(&self, identity: &Identity) -> Result<()>;
}

pub(crate) struct ReconnectingGateway {
    pub(crate) connection: Arc<dyn Connection>,
    pub(crate) agent: Arc<dyn Agent>,
    pub(crate) delay: Arc<dyn Delay>,
}

#[async_trait]
impl Gateway for ReconnectingGateway {
    async fn reconnect(&self, identity: &Identity) -> Result<()> {
        let mut seconds = 1;
        loop {
            match self.authenticate(identity).await {
                Ok(()) => return Ok(()),
                Err(Error::Revoked) => {
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
    async fn authenticate(&self, identity: &Identity) -> Result<()> {
        self.connection.connect().await?;
        self.connection
            .send(&AgentMessage::Hello {
                protocol_version: ProtocolVersion::CURRENT,
                workspace_id: identity.workspace_id.clone(),
                machine_id: identity.machine_id.clone(),
                credential: identity.credential.clone(),
                report: self.agent.report().await?,
            })
            .await?;
        match self.connection.receive_handshake().await? {
            ControlMessage::Welcome { protocol_version }
                if protocol_version == ProtocolVersion::CURRENT =>
            {
                Ok(())
            }
            message @ ControlMessage::Remove { .. } => {
                self.agent.control(message).await?;
                Err(Error::Revoked)
            }
            _ => Err(Error::ProtocolVersion),
        }
    }
}

#[cfg(test)]
#[path = "_tests_/gateway_tests.rs"]
mod gateway_tests;
