//! Independent machine-cache and Docker observations for dashboard readers.

use ciproxy_protocol::statistics::Statistics;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Statistics from one stored report, with independent section availability.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CacheView {
    /// Cache space and seven-day counts measured by the machine cache.
    pub machine_cache: Statistics<MachineCacheView>,
    /// Repository Docker disk use measured independently by the agent.
    pub docker_disks: Statistics<DockerDiskView>,
}

/// Measured machine-cache figures. Clear cache acts only on these entries.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct MachineCacheView {
    /// Machine-cache bytes used, as a decimal string.
    pub used_bytes: String,
    /// Machine-cache capacity bytes, as a decimal string.
    pub capacity_bytes: String,
    /// Handled restores during the last seven days, as a decimal string.
    pub restores_last_7_days: String,
    /// Local hits during the last seven days, as a decimal string.
    pub hits_last_7_days: String,
}

/// Measured persistent Docker disk figures, which Clear cache preserves.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct DockerDiskView {
    /// Repositories with persistent Docker disks, as a decimal string.
    pub repositories: String,
    /// Persistent Docker disk bytes, as a decimal string.
    pub used_bytes: String,
}
