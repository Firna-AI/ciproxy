//! Hosted API composition and blocking database query execution.

use std::sync::Arc;

use ciproxy_gateway::gateway::MachineGateway;
use ciproxy_github::app::GitHubApp;
use ciproxy_github::scale_sets::ScaleSetFactory;
use ciproxy_store_interface::store::Store;

use crate::access::DashboardAccess;
use crate::capacity::MachineCapacity;
use crate::error::{Error, ErrorResultExt, Result};
use crate::installations::InstallationAdministration;
use crate::runtime::Runtime;

/// Public HTTPS locations and the immutable API build identity.
#[derive(Clone)]
pub struct ApiConfig {
    /// Exact browser origin used by CSRF checks, without a trailing slash.
    pub public_origin: String,
    /// GitHub App installation URL.
    pub github_install_url: String,
    /// HTTPS install script URL for the environment.
    pub download_url: String,
    /// Current full image disk size from trusted release metadata.
    pub image_disk_bytes: u64,
    /// Build identity returned in every x-ciproxy-build response header.
    pub build: String,
}

/// Runtime collaborators constructed by the hosted control-plane owner.
#[derive(Clone)]
pub struct ApiState {
    /// Public locations and immutable build metadata.
    pub config: ApiConfig,
    /// Durable tenant-scoped PostgreSQL query boundary.
    pub store: Arc<dyn Store>,
    /// Shared sign-in and authorization service.
    pub access: Arc<dyn DashboardAccess>,
    /// The same machine gateway that owns agent WebSockets.
    pub gateway: Arc<dyn MachineGateway>,
    /// The same GitHub App service that owns webhook lifecycle delivery.
    pub github: Arc<dyn GitHubApp>,
    /// App-authenticated scale-set factory.
    pub scale_sets: Arc<dyn ScaleSetFactory>,
    /// App-JWT uninstall adapter.
    pub installations: Arc<dyn InstallationAdministration>,
    /// Actual capacity read from the exact prepared machine snapshot.
    pub capacity: Arc<dyn MachineCapacity>,
    /// Trusted server clock and entropy.
    pub runtime: Arc<dyn Runtime>,
}

pub(crate) async fn query<T: Send + 'static>(
    store: Arc<dyn Store>,
    action: impl FnOnce(&dyn Store) -> ciproxy_store_interface::error::Result<T> + Send + 'static,
) -> Result<T> {
    let result = tokio::task::spawn_blocking(move || action(store.as_ref()))
        .await
        .internal()?;
    match result {
        Ok(value) => Ok(value),
        Err(ciproxy_store_interface::error::Error::NotFound) => Err(Error::NotFound),
        Err(ciproxy_store_interface::error::Error::Conflict) => Err(Error::Conflict),
        Err(
            ciproxy_store_interface::error::Error::InvalidValue
            | ciproxy_store_interface::error::Error::InvalidReference,
        ) => Err(Error::InvalidInput),
        Err(source) => Err(Error::internal(source)),
    }
}
