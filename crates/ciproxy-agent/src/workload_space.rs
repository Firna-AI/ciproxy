//! Root and Docker reservations release the same cache and idle-base capacity.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::boundaries::{Cache, Host};
use crate::capacity::machine_resources;
use crate::docker::DockerDisks;
use crate::error::Result;
use crate::state::MachineState;

/// Callers hold the shared allocation lock through reservation and cache release.
#[cfg_attr(test, unimock::unimock(api = WorkloadSpaceMock))]
#[async_trait]
pub(crate) trait WorkloadSpace: Send + Sync {
    /// Read the current workload pool after all durable root reservations.
    async fn available(&self, image_budget: u64) -> Result<u64>;
    /// Reclaim idle Docker bases and return the cache's remaining allocation.
    fn cache_budget(&self, free_pool: u64, dockers: Option<&dyn DockerDisks>) -> Result<u64>;
    /// Complete reclamation and cache reduction before a job disk can write.
    async fn release(&self, free_pool: u64, dockers: Option<&dyn DockerDisks>) -> Result<()>;
}

/// Read shared machine state and apply the common disk release operation.
pub(crate) struct MachineWorkloadSpace {
    pub(crate) state: Arc<Mutex<MachineState>>,
    pub(crate) host: Arc<dyn Host>,
    pub(crate) cache: Arc<dyn Cache>,
}

#[async_trait]
impl WorkloadSpace for MachineWorkloadSpace {
    async fn available(&self, image_budget: u64) -> Result<u64> {
        let hardware = self.host.hardware()?;
        let state = self.state.lock().await;
        Ok(machine_resources(&hardware, image_budget)
            .available(state.runners.iter().map(|vm| vm.report.size))
            .job_disk_bytes)
    }

    fn cache_budget(&self, free_pool: u64, dockers: Option<&dyn DockerDisks>) -> Result<u64> {
        match dockers {
            Some(dockers) => {
                let free = free_pool.saturating_sub(dockers.report()?.leased_bytes);
                Ok(free.saturating_sub(dockers.reclaim(free)?))
            }
            None => Ok(free_pool),
        }
    }

    async fn release(&self, free_pool: u64, dockers: Option<&dyn DockerDisks>) -> Result<()> {
        self.cache
            .budget(self.cache_budget(free_pool, dockers)?)
            .await
    }
}
