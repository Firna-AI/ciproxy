//! Durable machine control with short state locks and replaceable host boundaries.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::reports::MachineReport;
use ciproxy_protocol::runners::RunnerState;
use tokio::sync::Mutex;

use crate::boundaries::{Cache, Host, Images, Store, Vms};
use crate::capacity::machine_resources;
use crate::error::Result;
use crate::state::{MachineState, Mode};

#[cfg_attr(test, unimock::unimock(api = AgentMock))]
#[async_trait]
pub(crate) trait Agent: Send + Sync {
    async fn restore(&self) -> Result<()>;
    async fn reconcile(&self) -> Result<()>;
    async fn report(&self) -> Result<MachineReport>;
    async fn control(&self, message: ControlMessage) -> Result<Option<AgentMessage>>;
    async fn guest(&self, runner_id: &str, message: GuestToAgent) -> Result<Option<AgentMessage>>;
    async fn remove(&self) -> Result<()>;
    async fn finished(&self) -> bool;
}

pub(crate) struct Controller {
    pub(crate) state: Mutex<MachineState>,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) host: Arc<dyn Host>,
    pub(crate) vms: Arc<dyn Vms>,
    pub(crate) images: Arc<dyn Images>,
    pub(crate) cache: Arc<dyn Cache>,
}

#[async_trait]
impl Agent for Controller {
    async fn reconcile(&self) -> Result<()> {
        let hardware = self.host.hardware()?;
        let owned = self.state.lock().await.runners.clone();
        for vm in owned {
            self.vms.restore(&vm).await?;
        }
        let (mode, budget) = {
            let state = self.state.lock().await;
            (
                state.mode,
                machine_resources(&hardware)
                    .available(state.runners.iter().map(|vm| vm.report.size))
                    .job_disk_bytes,
            )
        };
        if let Err(error) = self.cache.budget(budget).await {
            tracing::warn!(error = %error, "cache budget unavailable; keep jobs running");
        }
        if mode != Mode::Active {
            self.set_mode(mode).await?;
        } else {
            let stopping = self
                .state
                .lock()
                .await
                .runners
                .iter()
                .filter(|vm| vm.report.state == RunnerState::Stopping)
                .cloned()
                .collect::<Vec<_>>();
            for vm in stopping {
                self.request_idle_stop(&vm).await?;
            }
        }
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        let runners = self.state.lock().await.runners.clone();
        for vm in runners {
            self.vms.restore(&vm).await?;
        }
        let mode = self.state.lock().await.mode;
        if mode != Mode::Active {
            self.set_mode(mode).await?;
        }
        Ok(())
    }

    async fn report(&self) -> Result<MachineReport> {
        self.machine_report().await
    }

    async fn control(&self, message: ControlMessage) -> Result<Option<AgentMessage>> {
        self.apply(message).await
    }

    async fn guest(&self, runner_id: &str, message: GuestToAgent) -> Result<Option<AgentMessage>> {
        self.observe(runner_id, message).await
    }

    async fn remove(&self) -> Result<()> {
        self.set_mode(Mode::Removed).await
    }

    async fn finished(&self) -> bool {
        let state = self.state.lock().await;
        state.mode == Mode::Removed && state.runners.is_empty()
    }
}

#[cfg(test)]
#[path = "_tests_/controller_tests.rs"]
mod controller_tests;

#[cfg(test)]
#[path = "_tests_/drain_tests.rs"]
mod drain_tests;
