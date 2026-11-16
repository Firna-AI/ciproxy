//! Move a legacy endpoint into the atomic identity before any credential use.

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

impl Store for IdentityStore {
    fn load(&self) -> Result<MachineState> {
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

    fn save(&self, state: &MachineState) -> Result<()> {
        self.store.save(state)
    }

    fn clear(&self) -> Result<()> {
        self.store.clear()
    }
}

#[cfg(test)]
#[path = "_tests_/identity_store_tests.rs"]
mod identity_store_tests;
