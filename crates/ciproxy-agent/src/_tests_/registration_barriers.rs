//! Explicit VM operation barriers keep registration interleavings deterministic.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::runners::RunnerReport;
use tokio::sync::Notify;

use crate::boundaries::Vms;
use crate::error::Result;
use crate::state::VmRecord;

pub(super) enum Hold {
    Delivery,
    Stop,
}

pub(super) struct HeldVms {
    pub(super) delegate: Arc<dyn Vms>,
    pub(super) hold: Hold,
    pub(super) started: Arc<Notify>,
    pub(super) release: Arc<Notify>,
}

#[async_trait]
impl Vms for HeldVms {
    async fn networks(&self) -> Result<Vec<u16>> {
        self.delegate.networks().await
    }
    async fn configuration_lost(&self, vm: &VmRecord) -> Result<bool> {
        self.delegate.configuration_lost(vm).await
    }
    async fn clear_docker(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.clear_docker(vm).await
    }
    async fn allocate(&self, report: RunnerReport, used: &[u16]) -> Result<VmRecord> {
        self.delegate.allocate(report, used).await
    }
    async fn remove_network(&self, index: u16) -> Result<()> {
        self.delegate.remove_network(index).await
    }
    async fn start(&self, vm: &VmRecord, config: AgentToGuest) -> Result<()> {
        self.delegate.start(vm, config).await
    }
    async fn restore(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.restore(vm).await
    }
    async fn request_stop(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.request_stop(vm).await
    }
    async fn stop(&self, vm: &VmRecord) -> Result<()> {
        if matches!(self.hold, Hold::Stop) {
            self.started.notify_one();
            self.release.notified().await;
        }
        self.delegate.stop(vm).await
    }
    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()> {
        if matches!(self.hold, Hold::Delivery)
            && matches!(message, AgentToGuest::ConfigureRunner { .. })
        {
            self.started.notify_one();
            self.release.notified().await;
        }
        self.delegate.send(vm, message).await
    }
    async fn swap_docker(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.swap_docker(vm).await
    }
}
