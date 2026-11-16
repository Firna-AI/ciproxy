//! Stop the previous service before reading and migrating its durable identity.

use std::sync::Arc;

use thiserror::Error;

use crate::application_endpoint::{ConfiguredAction, configure};
use crate::boundaries::{Service, Store};
use crate::cli::Action;
use crate::endpoint::EndpointSettings;
use crate::error;
use crate::state::MachineState;

/// Distinguish state recovery advice from command and service failures.
#[derive(Debug, Error)]
pub(crate) enum InvocationError {
    /// Endpoint selection or stopping the previous service failed.
    #[error("[agent/setup] command or service setup failed")]
    Setup(#[source] error::Error),
    /// The caller supplies recovery advice for the durable identity.
    #[error("[agent/setup] machine identity state is unavailable")]
    State(#[source] error::Error),
}

/// A startup result whose state failures keep their existing recovery context.
pub(crate) type InvocationResult<T> = std::result::Result<T, InvocationError>;

/// A resolved command with the identity loaded after any required service stop.
pub(crate) struct PreparedInvocation {
    pub(crate) action: ConfiguredAction,
    pub(crate) state: MachineState,
}

/// Prepare one invocation before constructing any credential-bearing client.
pub(crate) trait InvocationSetup: Send + Sync {
    fn prepare(&self, action: Action) -> InvocationResult<PreparedInvocation>;
}

/// Compose command settings, service ownership, and identity migration.
pub(crate) struct LoadedInvocation {
    pub(crate) endpoints: Arc<dyn EndpointSettings>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) store: Arc<dyn Store>,
}

impl InvocationSetup for LoadedInvocation {
    fn prepare(&self, action: Action) -> InvocationResult<PreparedInvocation> {
        let action = match configure(action, self.endpoints.as_ref()) {
            Ok(action) => action,
            Err(error) => return Err(InvocationError::Setup(error)),
        };
        if !matches!(action, ConfiguredAction::Run { .. })
            && let Err(error) = self.service.stop()
        {
            return Err(InvocationError::Setup(error));
        }
        let state = match self.store.load() {
            Ok(state) => state,
            Err(error) => return Err(InvocationError::State(error)),
        };
        Ok(PreparedInvocation {
            action: action.bind_identity(state.identity.as_ref()),
            state,
        })
    }
}

#[cfg(test)]
#[path = "_tests_/application_setup_tests.rs"]
mod application_setup_tests;
