//! Registration happens once. Reinstall uses the existing root-only identity.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;

use crate::boundaries::{Connection, Host, Service, Store};
use crate::connection_types::ControlEvent;
use crate::controller::Agent;
use crate::endpoint::AgentEndpoint;
use crate::error::{Error, ErrorResultExt, Result};
use crate::host_preparation::Preparation;
use crate::state::{Identity, Mode};

#[async_trait]
pub(crate) trait Install: Send + Sync {
    async fn install(&self, token: Secret) -> Result<InstallOutcome>;
}

/// The completed registration or repair result printed by the CLI.
pub(crate) enum InstallOutcome {
    /// Registration or repair uses the endpoint selected by the command.
    Installed,
    /// Repair kept an identity whose endpoint differs from the command.
    Repaired {
        endpoint: Option<AgentEndpoint>,
        requested: AgentEndpoint,
    },
}

impl InstallOutcome {
    /// Produce the last CLI line after all install or repair work succeeds.
    pub(crate) fn message(&self) -> String {
        match self {
            Self::Installed => "ciproxy agent installed".to_owned(),
            Self::Repaired {
                endpoint,
                requested,
            } => format!(
                "ciproxy agent repaired. Machine stays at {}. Command selected {}. To move the machine, uninstall first.",
                endpoint
                    .as_ref()
                    .map_or("an unavailable legacy endpoint", AgentEndpoint::as_str),
                requested.as_str(),
            ),
        }
    }
}

pub(crate) struct Installer {
    pub(crate) endpoint: AgentEndpoint,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) host: Arc<dyn Host>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) connection: Arc<dyn Connection>,
    pub(crate) agent: Arc<dyn Agent>,
    pub(crate) preparation: Arc<dyn Preparation>,
}

#[async_trait]
impl Install for Installer {
    async fn install(&self, token: Secret) -> Result<InstallOutcome> {
        self.host.require_root()?;
        self.preparation.prepare().await?;
        let mut state = self.store.load()?;
        if state.mode != Mode::Removed
            && let Some(identity) = &state.identity
        {
            self.service.install()?;
            return Ok(if identity.endpoint.as_ref() == Some(&self.endpoint) {
                InstallOutcome::Installed
            } else {
                InstallOutcome::Repaired {
                    endpoint: identity.endpoint.clone(),
                    requested: self.endpoint.clone(),
                }
            });
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
            endpoint: Some(self.endpoint.clone()),
        });
        state.mode = Mode::Active;
        self.store.save(&state)?;
        self.service.install()?;
        Ok(InstallOutcome::Installed)
    }
}

#[cfg(test)]
#[path = "_tests_/installation_tests.rs"]
mod installation_tests;

#[cfg(test)]
#[path = "_tests_/endpoint_install_tests.rs"]
mod endpoint_install_tests;

#[cfg(test)]
#[path = "_tests_/endpoint_mismatch_tests.rs"]
mod endpoint_mismatch_tests;

#[cfg(test)]
#[path = "_tests_/legacy_install_tests.rs"]
mod legacy_install_tests;
