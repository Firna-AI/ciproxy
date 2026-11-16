//! Recover retained endpoints without retiring a live identity's legacy source.

use std::fmt::Debug;
use std::sync::Arc;

use crate::boundaries::Store;
use crate::endpoint::{AgentEndpoint, EndpointFiles};
use crate::error::Result;
use crate::state::MachineState;

/// Resolve legacy endpoints on reads and retire files when identity ends.
pub(crate) struct IdentityStore {
    pub(crate) store: Arc<dyn Store>,
    pub(crate) endpoints: Arc<dyn EndpointFiles>,
}

/// Proof that startup has excluded the previous runtime or stopped its service.
pub trait IdentityOwner: Debug + Send + Sync {}

/// Persist recovered identity only while the caller holds startup ownership.
#[cfg_attr(test, unimock::unimock(api = IdentityLoaderMock))]
pub(crate) trait IdentityLoader: Store {
    fn load_owned(&self, owner: &dyn IdentityOwner) -> Result<MachineState>;
}

impl IdentityLoader for IdentityStore {
    fn load_owned(&self, _owner: &dyn IdentityOwner) -> Result<MachineState> {
        let original = self.store.load()?;
        let state = self.restore(original.clone());
        if state != original {
            self.store.save(&state)?;
        }
        self.cleanup(&state);
        Ok(state)
    }
}

impl Store for IdentityStore {
    fn load(&self) -> Result<MachineState> {
        Ok(self.restore(self.store.load()?))
    }

    fn save(&self, state: &MachineState) -> Result<()> {
        self.store.save(state)?;
        self.cleanup(state);
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        self.store.clear()?;
        self.remove_legacy();
        Ok(())
    }
}

impl IdentityStore {
    fn legacy(&self) -> Result<Option<AgentEndpoint>> {
        self.endpoints
            .read()?
            .map(|value| value.parse())
            .transpose()
    }

    fn restore(&self, mut state: MachineState) -> MachineState {
        if let Some(identity) = state.identity.as_mut()
            && identity.endpoint.is_none()
        {
            match self.legacy() {
                Ok(Some(endpoint)) => identity.endpoint = Some(endpoint),
                Ok(None) => tracing::warn!(
                    "Legacy agent endpoint is missing. Keep local cleanup available without hosted connections."
                ),
                Err(error) => {
                    tracing::warn!(%error, "Legacy agent endpoint is unavailable. Keep local cleanup available without hosted connections.")
                }
            }
        }
        state
    }

    fn cleanup(&self, state: &MachineState) {
        let remove = match &state.identity {
            None => true,
            Some(identity) => identity.endpoint.as_ref().is_some_and(
                |endpoint| matches!(self.legacy(), Ok(Some(legacy)) if legacy != *endpoint),
            ),
        };
        if remove {
            self.remove_legacy();
        }
    }

    fn remove_legacy(&self) {
        if let Err(error) = self.endpoints.remove() {
            tracing::warn!(%error, "The identity no longer uses the legacy endpoint file. Its cleanup will retry at the next start.");
        }
    }
}

#[cfg(test)]
#[path = "_tests_/identity_store_tests.rs"]
mod identity_store_tests;
