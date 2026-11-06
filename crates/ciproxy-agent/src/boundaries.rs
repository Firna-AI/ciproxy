//! Mockable host, transport, storage, image, cache, and VM boundaries.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::cache::{CacheConnection, MachineCacheReport};
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::images::{ImageDownload, ImageReport};
use ciproxy_protocol::jobs::{JobFacts, JobScope};
use ciproxy_protocol::reports::HardwareReport;
use ciproxy_protocol::runners::RunnerReport;
use ciproxy_protocol::runtime::RuntimeCredentials;

use crate::connection_types::{ConnectionResult, ControlEvent};
use crate::error::Result;
use crate::image_registry::ImageLease;
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
    fn purge(&self) -> Result<()>;
    async fn fetch(&self, image: &ImageDownload) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = CacheMock))]
#[async_trait]
pub(crate) trait Cache: Send + Sync {
    async fn fallback(&self, vm: &VmRecord) -> Result<()>;
    async fn budget(&self, capacity_bytes: u64) -> Result<()>;
    async fn report(&self) -> Result<MachineCacheReport>;
    async fn clear(&self) -> Result<()>;
    async fn release(&self, vm: &VmRecord) -> Result<()>;
    async fn handoff(
        &self,
        vm: &VmRecord,
        job_id: &JobId,
        runtime: RuntimeCredentials,
    ) -> Result<()>;
    async fn prepare(
        &self,
        runner: &RunnerId,
        facts: &JobFacts,
    ) -> Result<Option<(CacheConnection, JobScope, u64)>>;
}

#[cfg_attr(test, unimock::unimock(api = VmsMock))]
#[async_trait]
pub(crate) trait Vms: Send + Sync {
    async fn clear_docker(&self, vm: &VmRecord) -> Result<()>;
    fn allocate(&self, report: RunnerReport, used: &[VmRecord]) -> Result<VmRecord>;
    async fn start(&self, vm: &VmRecord, config: AgentToGuest) -> Result<()>;
    async fn restore(&self, vm: &VmRecord) -> Result<()>;
    async fn request_stop(&self, vm: &VmRecord) -> Result<()>;
    async fn stop(&self, vm: &VmRecord) -> Result<()>;
    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()>;
    async fn swap_docker(&self, vm: &VmRecord) -> Result<()>;
}

#[derive(Debug)]
pub(crate) struct GuestEvent {
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
