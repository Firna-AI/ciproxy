//! Overflow providers and images available in this deployment.

use std::{collections::HashMap, sync::Arc};

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_store_interface::overflow::provider::OverflowProvider;

use crate::error::{Error, Result};
use crate::provider::ProviderFactory;

/// The exact deployed image and its disk geometry for one provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageSource {
    /// Provider-specific reference, such as a shared snapshot name or share code.
    pub reference: String,
    /// Root geometry of that same image release, shared with the scaler.
    pub root_sizing: RootSizing,
}

/// The factory and image of a provider enabled by the deployment.
pub struct ConfiguredProvider {
    /// Builds the provider capabilities and interprets its image reference.
    pub factory: Arc<dyn ProviderFactory>,
    /// Image to save with a new delivery.
    pub image: ImageSource,
}

/// Lookup for providers that have their deployment settings.
#[derive(Default)]
pub struct ProviderRegistry {
    providers: HashMap<OverflowProvider, ConfiguredProvider>,
}

impl ProviderRegistry {
    /// Whether the deployment has no provider configurations.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Register or replace the configuration of one available provider.
    pub fn with_provider(
        mut self,
        provider: OverflowProvider,
        factory: Arc<dyn ProviderFactory>,
        image: ImageSource,
    ) -> Self {
        self.providers
            .insert(provider, ConfiguredProvider { factory, image });
        self
    }

    /// Return the configured provider or the provider-unavailable error.
    pub fn get(&self, provider: OverflowProvider) -> Result<&ConfiguredProvider> {
        self.providers
            .get(&provider)
            .ok_or(Error::ProviderUnavailable)
    }
}
