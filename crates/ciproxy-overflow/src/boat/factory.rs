//! Production capability composition with one verified HTTPS origin and wallet.

use std::{collections::BTreeMap, sync::Arc};

use ciproxy_protocol::secret::Secret;
use sandbox_boat::control::client::ReqwestBoatControlApi;
use sandbox_boat::runtime::client::HttpsBoatRuntimeApi;
use sandbox_boat::runtime::configured::BoatTrustedRuntime;
use sandbox_boat::transport::http::ReqwestBoatHttpTransport;
use sandbox_boat::transport::settings::BoatHttpSettings;
use sandbox_boat::{BoatAdapterConfig, BoatProfile, BoatSandboxBackend};

use crate::error::{Result, ResultExt};
use crate::maintenance::MaintenanceCapabilities;
use crate::provider::{BackendFactory, Capabilities};

use super::account::AccountApi;
use super::http::BoatReads;
use super::launcher::ImageLauncher;

/// Composition root for the reviewed Boat adapter and fixed ciproxy image.
pub struct BoatFactory {
    origin: String,
    extra_root_pem: Option<Vec<u8>>,
}

impl Default for BoatFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BoatFactory {
    /// Use Boat's production HTTPS endpoint and system trust roots.
    pub fn new() -> Self {
        Self {
            origin: "https://boat.dev".to_owned(),
            extra_root_pem: None,
        }
    }

    /// Select an HTTPS deployment and additional trust root without disabling TLS checks.
    pub fn with_origin(origin: String, extra_root_pem: Option<Vec<u8>>) -> Self {
        Self {
            origin,
            extra_root_pem,
        }
    }
}

impl BackendFactory for BoatFactory {
    fn backend(&self, key: &Secret, wallet: &str) -> Result<Capabilities> {
        let profiles = BTreeMap::from([(
            "ciproxy".to_owned(),
            BoatProfile {
                provider_size: "default".to_owned(),
                allow_public_egress: true,
                denied_destinations: Vec::new(),
            },
        )]);
        let config = BoatAdapterConfig::new("boat", &self.origin, key.expose_secret(), profiles)
            .internal()?
            .with_organization(wallet)
            .internal()?;
        let transport = Arc::new(
            ReqwestBoatHttpTransport::with_settings(
                config.clone(),
                BoatHttpSettings::default(),
                self.extra_root_pem.as_deref(),
            )
            .internal()?,
        );
        let runtime = Arc::new(HttpsBoatRuntimeApi::new(transport.clone()));
        let backend = Arc::new(BoatSandboxBackend::with_clients(
            config.clone(),
            Arc::new(ReqwestBoatControlApi::with_transport(transport)),
            Arc::new(BoatTrustedRuntime::new(config.clone(), runtime.clone())),
        ));
        let reads = BoatReads::new(
            config.api_base().to_owned(),
            key.clone(),
            wallet.to_owned(),
            self.extra_root_pem.as_deref(),
        )?;
        let launcher = Arc::new(ImageLauncher { runtime });
        Ok(Capabilities {
            lifecycle: backend.clone(),
            capacity: backend.clone(),
            files: backend,
            account: Arc::new(AccountApi { reads }),
            launcher: launcher.clone(),
            inspection: launcher,
        })
    }

    fn maintenance(&self, key: &Secret, wallet: &str) -> Result<MaintenanceCapabilities> {
        Ok(self.backend(key, wallet)?.maintenance())
    }
}
