//! Durable overflow execution under one exclusive workspace session.

mod accounts;
mod adoption;
mod authority;
mod cleanup;
mod costs;
mod execution;
mod inventory;
mod launch;
mod preparation;
mod provider_preparation;
mod refresh;
mod retired;
mod start;
mod usage;

use std::sync::Arc;

use ciproxy_store_interface::overflow::store::OverflowStore;

use crate::api::{OverflowConfig, RunnerAuthority};
use crate::clock::Clock;
use crate::crypto::KeyCipher;
use crate::registry::ProviderRegistry;
use crate::secrets::TokenSource;

/// Dependencies constructed by the hosted control-plane composition root.
pub struct OverflowDependencies {
    /// Synchronous store. Calls run on blocking workers inside the service.
    pub store: Arc<dyn OverflowStore>,
    /// Real Cloud KMS encryption in production.
    pub cipher: Arc<dyn KeyCipher>,
    /// Configured provider factories and the images they use for new starts.
    pub providers: ProviderRegistry,
    /// Current queue and GitHub registration authority.
    pub runners: Arc<dyn RunnerAuthority>,
    /// UTC clock with a deterministic unit-test mock.
    pub clock: Arc<dyn Clock>,
    /// Fresh sandbox callback entropy.
    pub tokens: Arc<dyn TokenSource>,
}

/// Production implementation of the object-safe overflow boundary.
pub struct OverflowService {
    config: OverflowConfig,
    dependencies: OverflowDependencies,
}

impl OverflowService {
    /// Inject runtime boundaries and the deployment configuration.
    pub fn new(config: OverflowConfig, dependencies: OverflowDependencies) -> Self {
        Self {
            config,
            dependencies,
        }
    }
}
