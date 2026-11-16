//! Move a legacy endpoint into the atomic identity before any credential use.

use std::fmt::Debug;
use std::sync::Arc;

use crate::boundaries::Store;
use crate::endpoint::{AgentEndpoint, EndpointFiles};
use crate::error::Result;
use crate::state::MachineState;

/// Add the legacy endpoint once while preserving one atomic identity record.
pub(crate) struct IdentityStore {
    pub(crate) store: Arc<dyn Store>,
    pub(crate) endpoints: Arc<dyn EndpointFiles>,
}

/// Proof that startup has excluded the previous runtime or stopped its service.
pub trait IdentityOwner: Debug + Send + Sync {}

/// Load and migrate identity only while the caller holds startup ownership.
#[cfg_attr(test, unimock::unimock(api = IdentityLoaderMock))]
pub(crate) trait IdentityLoader: Send + Sync {
    fn load(&self, owner: &dyn IdentityOwner) -> Result<MachineState>;
}

impl IdentityLoader for IdentityStore {
    fn load(&self, _owner: &dyn IdentityOwner) -> Result<MachineState> {
        let mut state = self.store.load()?;
        if let Some(identity) = state.identity.as_mut()
            && identity.endpoint.is_none()
        {
            let endpoint = self.endpoints.read().and_then(|value| {
                value
                    .map(|value| value.parse::<AgentEndpoint>())
                    .transpose()
            });
            match endpoint {
                Ok(Some(endpoint)) => identity.endpoint = Some(endpoint),
                Ok(None) => {
                    tracing::warn!(
                        "Legacy agent endpoint is missing. Keep local cleanup available without hosted connections."
                    );
                    return Ok(state);
                }
                Err(error) => {
                    tracing::warn!(%error, "Legacy agent endpoint is unavailable. Keep local cleanup available without hosted connections.");
                    return Ok(state);
                }
            }
            self.store.save(&state)?;
        }
        if let Err(error) = self.endpoints.remove() {
            tracing::warn!(%error, "The identity no longer uses the legacy endpoint file. Its cleanup will retry at the next start.");
        }
        Ok(state)
    }
}

#[cfg(test)]
#[path = "_tests_/identity_store_tests.rs"]
mod identity_store_tests;
