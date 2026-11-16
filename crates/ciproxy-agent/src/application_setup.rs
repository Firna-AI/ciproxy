//! Inspect install identity or acquire ownership before identity migration.

use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use thiserror::Error;

use crate::application_endpoint::{ConfiguredAction, configure};
use crate::boundaries::Service;
use crate::cli::Action;
use crate::endpoint::EndpointSettings;
use crate::error;
use crate::identity_store::{IdentityLoader, IdentityOwner};
use crate::runtime_ownership::{RuntimeClaims, RuntimeOwner};
use crate::state::MachineState;

/// Distinguish state recovery advice from command and service failures.
#[derive(Debug, Error)]
pub(crate) enum InvocationError {
    /// Endpoint selection, runtime claim, or service stop failed.
    #[error("[agent/setup] command or service setup failed")]
    Setup(#[source] error::Error),
    /// The caller supplies recovery advice for the durable identity.
    #[error("[agent/setup] machine identity state is unavailable")]
    State(#[source] error::Error),
}

/// A startup result whose state failures keep their existing recovery context.
pub(crate) type InvocationResult<T> = std::result::Result<T, InvocationError>;

/// A resolved command and identity with optional migration ownership.
pub(crate) struct PreparedInvocation {
    pub(crate) action: ConfiguredAction,
    pub(crate) state: MachineState,
    pub(crate) ownership: Option<InvocationOwnership>,
}

/// Hold a runtime claim or stopped-service proof through identity recovery.
pub(crate) struct InvocationOwnership {
    runtime: Option<Arc<dyn RuntimeOwner>>,
}

impl Debug for InvocationOwnership {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InvocationOwnership")
            .field("runtime", &self.runtime.is_some())
            .finish()
    }
}

impl IdentityOwner for InvocationOwnership {}

impl InvocationOwnership {
    /// Share the same file claim with image recovery and its runtime owner.
    pub(crate) fn runtime(&self) -> Option<Arc<dyn RuntimeOwner>> {
        self.runtime.clone()
    }
}

/// Prepare one invocation before constructing any credential-bearing client.
pub(crate) trait InvocationSetup: Send + Sync {
    fn prepare(&self, action: Action) -> InvocationResult<PreparedInvocation>;
}

/// Compose command settings, startup ownership, and identity migration.
pub(crate) struct LoadedInvocation {
    pub(crate) endpoints: Arc<dyn EndpointSettings>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) identity: Arc<dyn IdentityLoader>,
    pub(crate) runtime: Arc<dyn RuntimeClaims>,
}

impl InvocationSetup for LoadedInvocation {
    fn prepare(&self, action: Action) -> InvocationResult<PreparedInvocation> {
        let action = match configure(action, self.endpoints.as_ref()) {
            Ok(action) => action,
            Err(error) => return Err(InvocationError::Setup(error)),
        };
        let ownership = match action {
            ConfiguredAction::Run { .. } => match self.runtime.claim() {
                Ok(owner) => Some(InvocationOwnership {
                    runtime: Some(owner),
                }),
                Err(error) => return Err(InvocationError::Setup(error)),
            },
            ConfiguredAction::Uninstall => {
                if let Err(error) = self.service.stop() {
                    return Err(InvocationError::Setup(error));
                }
                Some(InvocationOwnership { runtime: None })
            }
            ConfiguredAction::Install { .. } => None,
        };
        let loaded = match &ownership {
            Some(owner) => self.identity.load_owned(owner),
            None => self.identity.load(),
        };
        let state = match loaded {
            Ok(state) => state,
            Err(error) => return Err(InvocationError::State(error)),
        };
        Ok(PreparedInvocation {
            action: action.bind_identity(state.identity.as_ref()),
            state,
            ownership,
        })
    }
}

#[cfg(test)]
#[path = "_tests_/application_setup_tests.rs"]
mod application_setup_tests;

#[cfg(test)]
#[path = "_tests_/run_ownership_tests.rs"]
mod run_ownership_tests;
