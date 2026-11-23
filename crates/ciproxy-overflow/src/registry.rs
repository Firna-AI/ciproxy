//! Overflow provider adapters and optional images for new starts.

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

/// A deployed maintenance adapter and its optional start image.
pub struct ConfiguredProvider {
    /// Builds the provider capabilities and interprets its image reference.
    pub factory: Arc<dyn ProviderFactory>,
    image: Option<ImageSource>,
}

/// Lookup for deployed adapters, including providers that can only do maintenance.
#[derive(Default)]
pub struct ProviderRegistry {
    providers: HashMap<OverflowProvider, ConfiguredProvider>,
}

impl ProviderRegistry {
    /// Register an adapter and enable new starts when an image is supplied.
    pub fn with_provider(
        mut self,
        provider: OverflowProvider,
        factory: Arc<dyn ProviderFactory>,
        image: Option<ImageSource>,
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

    /// Return the image for new starts when the provider is available.
    pub fn start_image(&self, provider: OverflowProvider) -> Result<&ImageSource> {
        self.get(provider)?
            .image
            .as_ref()
            .ok_or(Error::ProviderUnavailable)
    }
}
