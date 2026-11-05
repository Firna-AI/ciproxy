//! Durable Boat execution under one exclusive workspace session.

mod accounts;
mod cleanup;
mod execution;
mod launch;
mod refresh;
mod start;
mod usage;

use std::sync::Arc;

use ciproxy_store_interface::overflow::store::OverflowStore;

use crate::api::{OverflowConfig, RunnerAuthority};
use crate::clock::Clock;
use crate::crypto::KeyCipher;
use crate::provider::BackendFactory;
use crate::secrets::TokenSource;

/// Dependencies constructed by the hosted control-plane composition root.
pub struct OverflowDependencies {
    /// Synchronous store. Calls run on blocking workers inside the service.
    pub store: Arc<dyn OverflowStore>,
    /// Real Cloud KMS encryption in production.
    pub cipher: Arc<dyn KeyCipher>,
    /// The pinned Boat adapter and narrow account/launch operations.
    pub factory: Arc<dyn BackendFactory>,
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
    /// Inject all impure boundaries and the current shared image configuration.
    pub fn new(config: OverflowConfig, dependencies: OverflowDependencies) -> Self {
        Self {
            config,
            dependencies,
        }
    }
}
