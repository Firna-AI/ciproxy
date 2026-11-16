//! Cache and metadata adapters constructed by the Linux composition root.

use std::sync::Arc;

use crate::application_metadata::MetadataTarget;
use crate::cache_composition::{self, CacheServices};
use crate::cache_install::{CheckedCacheRelease, LinuxCacheInstallFiles};
use crate::cache_network::GuardedCacheStart;
use crate::cache_service::{CacheService, ReleasedCache};
use crate::cache_unit::{InstalledCacheUnit, LinuxCacheUnitFiles};
use crate::endpoint::AgentEndpoint;
use crate::error::Result;
use crate::process::Commands;
use crate::state::Identity;

/// Compose cache administration and its root metadata task for this identity.
pub(crate) fn prepare(
    commands: Arc<dyn Commands>,
    endpoint: Option<&AgentEndpoint>,
    identity: Option<&Identity>,
) -> Result<CacheServices> {
    let hosted = MetadataTarget::for_machine(endpoint, identity).client()?;
    cache_composition::services(commands, hosted)
}

/// Compose the checked cache release installer with the configured origin.
pub(crate) fn installer(commands: Arc<dyn Commands>) -> Arc<dyn CacheService> {
    Arc::new(ReleasedCache {
        unit: Arc::new(InstalledCacheUnit {
            files: Arc::new(LinuxCacheUnitFiles),
        }),
        starter: Arc::new(GuardedCacheStart {
            commands: commands.clone(),
        }),
        commands,
        files: Arc::new(LinuxCacheInstallFiles),
        release: Arc::new(CheckedCacheRelease {
            release_server: std::env::var("CIPROXY_RELEASE_SERVER_URL")
                .unwrap_or_else(|_| "https://get.ciproxy.dev".to_owned()),
        }),
    })
}
