//! Mockable host, transport, storage, image, cache, and VM boundaries.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::cache::{
    CacheConnection, DockerDiskAttachment, DockerDiskReport, MachineCacheReport,
};
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::images::{ImageDownload, ImageMetadata, ImageReport};
use ciproxy_protocol::jobs::{JobFacts, JobScope};
use ciproxy_protocol::reports::HardwareReport;
use ciproxy_protocol::runners::RunnerReport;

use crate::cache_authority::CheckedRuntime;
use crate::connection_types::{ConnectionResult, ControlEvent};
use crate::error::Result;
use crate::state::{MachineState, VmRecord};

#[cfg_attr(test, unimock::unimock(api = StoreMock))]
pub(crate) trait Store: Send + Sync {
    fn load(&self) -> Result<MachineState>;
    fn save(&self, state: &MachineState) -> Result<()>;
    fn clear(&self) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = HostMock))]
pub(crate) trait Host: Send + Sync {
    fn require_root(&self) -> Result<()>;
    fn hostname(&self) -> Result<String>;
    fn hardware(&self) -> Result<HardwareReport>;
}

#[cfg_attr(test, unimock::unimock(api = ServiceMock))]
pub(crate) trait Service: Send + Sync {
    fn install(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = ConnectionMock))]
#[async_trait]
pub(crate) trait Connection: Send + Sync {
    async fn connect(&self) -> ConnectionResult<()>;
    async fn send(&self, message: &AgentMessage) -> ConnectionResult<()>;
    async fn receive(&self) -> ConnectionResult<ControlEvent>;
    async fn receive_handshake(&self) -> ConnectionResult<ControlEvent>;
}

#[cfg_attr(test, unimock::unimock(api = ImagesMock))]
#[async_trait]
pub(crate) trait Images: Send + Sync {
    fn report(&self) -> Result<ImageReport>;
    fn recover(&self) -> Result<()>;
    fn hold_newest(&self) -> Result<Arc<dyn ImageLease>>;
    async fn fetch(&self, image: &ImageDownload) -> Result<()>;
}

/// A preparation hold keeps its selected image until its last reference ends.
#[cfg_attr(test, unimock::unimock(api = ImageLeaseMock))]
pub trait ImageLease: Send + Sync {
    /// Return the exact selected release and its logical root length.
    fn metadata(&self) -> ImageMetadata;
}

#[cfg_attr(test, unimock::unimock(api = DockerDisksMock))]
#[async_trait]
/// Durable Docker reservations and disk attachment operations.
pub trait DockerDisks: Send + Sync {
    fn reserved(&self, vm: &VmRecord) -> Result<bool>;
    fn offered(&self, vm: &VmRecord) -> Result<()>;
    fn failed(&self, vm: &VmRecord) -> Result<bool>;
    fn pending(&self, vm: &VmRecord) -> Result<bool>;
    async fn discard(&self, vm: &VmRecord) -> Result<()>;
    async fn prepare(
        &self,
        vm: &VmRecord,
        facts: &JobFacts,
        scope: &JobScope,
        data_bytes: u64,
        spare: u64,
    ) -> Result<Option<DockerDiskAttachment>>;
    fn release(&self, vm: &VmRecord) -> Result<()>;
    fn report(&self) -> Result<DockerDiskReport>;
    fn unused_bytes(&self) -> Result<u64>;
    fn reclaim(&self, free_pool: u64) -> Result<u64>;
}

#[cfg_attr(test, unimock::unimock(api = CacheMock))]
#[async_trait]
pub(crate) trait Cache: Send + Sync {
    async fn refresh_budget(&self, capacity_bytes: u64) -> Result<()>;
    async fn fallback(&self, vm: &VmRecord) -> Result<()>;
    async fn budget(&self, capacity_bytes: u64) -> Result<()>;
    async fn report(&self) -> Result<MachineCacheReport>;
    async fn clear(&self) -> Result<()>;
    async fn release(&self, vm: &VmRecord) -> Result<()>;
    async fn handoff(&self, vm: &VmRecord, job_id: &JobId, runtime: CheckedRuntime) -> Result<()>;
    async fn prepare(
        &self,
        runner: &RunnerId,
        facts: &JobFacts,
    ) -> Result<Option<(CacheConnection, JobScope, u64)>>;
}

#[cfg_attr(test, unimock::unimock(api = VmsMock))]
#[async_trait]
pub(crate) trait Vms: Send + Sync {
    /// List network indices whose reserved names remain on the host.
    async fn networks(&self) -> Result<Vec<u16>>;
    async fn configuration_lost(&self, vm: &VmRecord) -> Result<bool>;
    async fn clear_docker(&self, vm: &VmRecord) -> Result<()>;
    /// Allocate a VM identity while skipping active and pending network indices.
    async fn allocate(&self, report: RunnerReport, used: &[u16]) -> Result<VmRecord>;
    /// Retry network cleanup independently of VM and disk removal.
    async fn remove_network(&self, index: u16) -> Result<()>;
    async fn start(&self, vm: &VmRecord, config: AgentToGuest) -> Result<()>;
    /// Connect again to a VM that kept running across an agent restart.
    async fn reattach(&self, vm: &VmRecord) -> Result<()>;
    async fn request_stop(&self, vm: &VmRecord) -> Result<()>;
    async fn stop(&self, vm: &VmRecord) -> Result<()>;
    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()>;
    async fn swap_docker(&self, vm: &VmRecord) -> Result<()>;
}

#[derive(Debug)]
pub(crate) struct GuestEvent {
    pub(crate) provenance: GuestProvenance,
    pub(crate) configuration_lost: bool,
    pub(crate) runner_id: RunnerId,
    pub(crate) message: GuestToAgent,
}

#[cfg_attr(test, unimock::unimock(api = UpdatesMock))]
#[async_trait]
pub(crate) trait Updates: Send + Sync {
    async fn check(&self) -> Result<bool>;
    async fn restart(&self) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = DelayMock))]
#[async_trait]
pub(crate) trait Delay: Send + Sync {
    async fn wait(&self, seconds: u64);
}

/// Host connection identity that guest JSON cannot grant or restore.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuestProvenance {
    /// The original connection carried the fresh runner configuration.
    InitialConfiguration,
    /// Restored and reconnected channels have no first-handoff authority.
    Recovery,
}
