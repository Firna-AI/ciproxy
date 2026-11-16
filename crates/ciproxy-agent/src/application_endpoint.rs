//! Resolve CLI intent into control-plane targets for this root invocation.

use std::sync::Arc;

use ciproxy_protocol::secret::Secret;

use crate::boundaries::Connection;
use crate::cli::Action;
use crate::connection::WebSocketConnection;
use crate::connection_socket::TlsWebSocketDialer;
use crate::endpoint::{AgentEndpoint, EndpointSettings};
use crate::error::Result;

/// A lifecycle action with a required or optional control-plane target.
pub(crate) enum ConfiguredAction {
    /// Register or repair the service with its selected endpoint.
    Install {
        token: Secret,
        endpoint: AgentEndpoint,
    },
    /// Run the daemon against its selected endpoint.
    Run { endpoint: AgentEndpoint },
    /// Drain locally even when no trusted hosted target remains.
    Uninstall { endpoint: Option<AgentEndpoint> },
}

impl ConfiguredAction {
    /// Return the hosted target permitted for this lifecycle operation.
    pub(crate) fn endpoint(&self) -> Option<&AgentEndpoint> {
        match self {
            Self::Install { endpoint, .. } | Self::Run { endpoint } => Some(endpoint),
            Self::Uninstall { endpoint } => endpoint.as_ref(),
        }
    }
}

/// Resolve normal settings for the daemon and only the saved route for uninstall.
pub(crate) fn configure(
    action: Action,
    settings: &dyn EndpointSettings,
) -> Result<ConfiguredAction> {
    Ok(match action {
        Action::Install { token, agent_url } => ConfiguredAction::Install {
            token,
            endpoint: settings.resolve(agent_url.as_ref())?,
        },
        Action::Run => ConfiguredAction::Run {
            endpoint: settings.resolve(None)?,
        },
        Action::Uninstall => {
            let endpoint = match settings.saved() {
                Ok(endpoint) => endpoint,
                Err(error) => {
                    tracing::warn!(%error, "Saved agent endpoint is unavailable. Uninstall will continue without hosted cache metadata.");
                    None
                }
            };
            ConfiguredAction::Uninstall { endpoint }
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

#[cfg(test)]
#[path = "_tests_/application_endpoint_tests.rs"]
mod application_endpoint_tests;
