//! Resolve CLI intent into control-plane targets for this root invocation.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::secret::Secret;

use crate::boundaries::Connection;
use crate::cli::Action;
use crate::connection::WebSocketConnection;
use crate::connection_socket::TlsWebSocketDialer;
use crate::connection_types::{ConnectionError, ConnectionResult, ControlEvent};
use crate::endpoint::{AgentEndpoint, EndpointSettings};
use crate::error::Result;
use crate::state::Identity;

/// A lifecycle action with a required or optional control-plane target.
pub(crate) enum ConfiguredAction {
    /// Register or repair the service with its selected endpoint.
    Install {
        token: Secret,
        endpoint: AgentEndpoint,
    },
    /// Run the daemon against the route held with its credential.
    Run { endpoint: Option<AgentEndpoint> },
    /// Drain locally even when no trusted hosted target remains.
    Uninstall,
}

impl ConfiguredAction {
    /// Bind the daemon only after startup loads the retained identity.
    pub(crate) fn bind_identity(self, identity: Option<&Identity>) -> Self {
        match self {
            Self::Run { .. } => Self::Run {
                endpoint: identity.and_then(|identity| identity.endpoint.clone()),
            },
            action => action,
        }
    }
}

/// Resolve the command endpoint before loading a retained machine identity.
pub(crate) fn configure(
    action: Action,
    settings: &dyn EndpointSettings,
) -> Result<ConfiguredAction> {
    Ok(match action {
        Action::Install { token, agent_url } => ConfiguredAction::Install {
            token,
            endpoint: settings.resolve(agent_url.as_ref())?,
        },
        Action::Run => ConfiguredAction::Run { endpoint: None },
        Action::Uninstall => ConfiguredAction::Uninstall,
        // `application::main` runs the VM helper before it configures an action.
        #[cfg(target_os = "macos")]
        Action::VmRun { .. } => {
            return Err(crate::error::Error::VmHelper {
                reason: "the VM helper has no machine endpoint".to_owned(),
            });
        }
    })
}

/// Construct a control socket only for install or daemon operations.
pub(crate) fn connection(endpoint: &AgentEndpoint) -> Arc<dyn Connection> {
    Arc::new(WebSocketConnection::new(
        endpoint.as_str().to_owned(),
        Arc::new(TlsWebSocketDialer),
    ))
}

/// Keep jobs and local cleanup alive when a legacy route is unavailable.
pub(crate) fn retained_connection(endpoint: Option<&AgentEndpoint>) -> Arc<dyn Connection> {
    match endpoint {
        Some(endpoint) => connection(endpoint),
        None => Arc::new(UnavailableConnection),
    }
}

struct UnavailableConnection;

#[async_trait]
impl Connection for UnavailableConnection {
    async fn connect(&self) -> ConnectionResult<()> {
        Err(ConnectionError::Disconnected)
    }

    async fn send(&self, _message: &AgentMessage) -> ConnectionResult<()> {
        Err(ConnectionError::Disconnected)
    }

    async fn receive(&self) -> ConnectionResult<ControlEvent> {
        Err(ConnectionError::Disconnected)
    }

    async fn receive_handshake(&self) -> ConnectionResult<ControlEvent> {
        Err(ConnectionError::Disconnected)
    }
}

#[cfg(test)]
#[path = "_tests_/application_endpoint_tests.rs"]
mod application_endpoint_tests;
