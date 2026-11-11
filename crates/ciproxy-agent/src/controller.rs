//! Durable machine control with short state locks and replaceable host boundaries.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::reports::MachineReport;
use ciproxy_protocol::runners::RunnerState;
use tokio::sync::Mutex;

use crate::boundaries::{Cache, GuestProvenance, Host, Images, Store, Vms};
use crate::capacity::machine_resources;
use crate::docker::DockerDisks;
use crate::error::{Error, Result};
use crate::mode::ModeAction;
use crate::state::{MachineState, Mode};
use crate::terminal_completion::{CompletionAction, UninstallCleanup};

#[cfg_attr(test, unimock::unimock(api = AgentMock))]
#[async_trait]
pub(crate) trait Agent: Send + Sync {
    /// Check terminal networks and finish local cleanup before forgetting identity.
    async fn complete_removal(&self, action: CompletionAction) -> Result<bool>;
    async fn control_connection(&self, connected: bool) -> Result<()>;
    async fn configuration_lost(&self, runner_id: &str) -> Result<()>;
    async fn restore(&self) -> Result<()>;
    async fn reconcile(&self) -> Result<()>;
    async fn report(&self) -> Result<MachineReport>;
    async fn control(&self, message: ControlMessage) -> Result<Option<AgentMessage>>;
    async fn guest(
        &self,
        provenance: GuestProvenance,
        runner_id: &str,
        message: GuestToAgent,
    ) -> Result<Option<AgentMessage>>;
    async fn remove(&self) -> Result<()>;
}

pub(crate) struct Controller {
    /// Serialize admission while host inventory runs outside the state lock.
    pub(crate) allocation: Mutex<()>,
    /// Prevent overlapping cleanup attempts from deleting a reused index.
    pub(crate) network_cleanup: Mutex<()>,
    pub(crate) control_connected: AtomicBool,
    pub(crate) state: Mutex<MachineState>,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) host: Arc<dyn Host>,
    pub(crate) vms: Arc<dyn Vms>,
    pub(crate) images: Arc<dyn Images>,
    pub(crate) cache: Arc<dyn Cache>,
    pub(crate) image_budget: u64,
    pub(crate) dockers: Option<Arc<dyn DockerDisks>>,
}

#[async_trait]
impl Agent for Controller {
    async fn complete_removal(&self, action: CompletionAction) -> Result<bool> {
        let removed_empty = {
            let state = self.state.lock().await;
            state.mode == Mode::Removed && state.runners.is_empty()
        };
        if !removed_empty {
            return Ok(false);
        }
        self.retry_network_cleanup().await?;
        let uninstall = matches!(action, CompletionAction::Uninstall(_));
        if let CompletionAction::Uninstall(UninstallCleanup {
            preparation,
            service,
        }) = action
        {
            preparation.remove().await?;
            service.uninstall()?;
        }
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        next.identity = None;
        if uninstall {
            self.store.clear()?;
        } else if next != *state {
            self.store.save(&next)?;
        }
        *state = next;
        Ok(true)
    }

    async fn control_connection(&self, connected: bool) -> Result<()> {
        self.control_connected.store(connected, Ordering::Release);
        if !connected {
            self.release_pending_jobs().await?;
        }
        Ok(())
    }
    async fn configuration_lost(&self, runner_id: &str) -> Result<()> {
        let vm = self
            .state
            .lock()
            .await
            .runners
            .iter()
            .find(|vm| vm.report.runner_id.0 == runner_id)
            .cloned()
            .ok_or(Error::GuestMismatch)?;
        let lost = self.vms.configuration_lost(&vm).await?;
        if lost {
            self.record_configuration(runner_id, true).await?;
        }
        Ok(())
    }
    async fn reconcile(&self) -> Result<()> {
        let hardware = self.host.hardware()?;
        let owned = self.state.lock().await.runners.clone();
        for vm in owned {
            if self.state.lock().await.failed_starts.contains(&vm.local_id)
                && !matches!(vm.report.state, RunnerState::Running { .. })
            {
                if let Err(error) = self.retire(&vm).await {
                    tracing::warn!(%error, "failed-start VM cleanup remains pending");
                }
                continue;
            }
            self.vms.restore(&vm).await?;
            if let Err(error) = self.recover_docker(&vm).await {
                tracing::warn!(%error, "pending Docker recovery will retry");
            }
        }
        let budget = {
            let state = self.state.lock().await;
            machine_resources(&hardware, self.image_budget)
                .available(state.runners.iter().map(|vm| vm.report.size))
                .job_disk_bytes
        };
        if let Err(error) = self.cache.refresh_budget(self.cache_budget(budget)?).await {
            tracing::warn!(error = %error, "cache budget unavailable; keep jobs running");
        }
        self.enforce_mode().await?;
        self.retry_network_cleanup().await
    }

    async fn restore(&self) -> Result<()> {
        let runners = self.state.lock().await.runners.clone();
        for vm in runners {
            if self.state.lock().await.failed_starts.contains(&vm.local_id)
                && !matches!(vm.report.state, RunnerState::Running { .. })
            {
                continue;
            }
            self.vms.restore(&vm).await?;
            if let Err(error) = self.recover_docker(&vm).await {
                tracing::warn!(%error, "pending Docker recovery will retry");
            }
        }
        self.enforce_mode().await
    }

    async fn report(&self) -> Result<MachineReport> {
        self.machine_report().await
    }

    async fn control(&self, message: ControlMessage) -> Result<Option<AgentMessage>> {
        self.apply(message).await
    }

    async fn guest(
        &self,
        provenance: GuestProvenance,
        runner_id: &str,
        message: GuestToAgent,
    ) -> Result<Option<AgentMessage>> {
        self.observe(provenance, runner_id, message).await
    }

    async fn remove(&self) -> Result<()> {
        self.change_mode(ModeAction::Remove).await
    }
}

#[cfg(test)]
#[path = "_tests_/controller_tests.rs"]
mod controller_tests;

#[cfg(test)]
#[path = "_tests_/drain_tests.rs"]
mod drain_tests;

#[cfg(test)]
#[path = "_tests_/startup_failure_tests.rs"]
mod startup_failure_tests;

#[cfg(test)]
#[path = "_tests_/slot_allocation_tests.rs"]
mod slot_allocation_tests;

#[cfg(test)]
#[path = "_tests_/docker_admission_tests.rs"]
mod docker_admission_tests;

#[cfg(test)]
#[path = "_tests_/docker_policy_tests.rs"]
mod docker_policy_tests;

#[cfg(test)]
#[path = "_tests_/configuration_state_tests.rs"]
mod configuration_state_tests;

#[cfg(test)]
#[path = "_tests_/unconfigured_retirement_tests.rs"]
mod unconfigured_retirement_tests;

#[cfg(test)]
#[path = "_tests_/startup_removal_tests.rs"]
mod startup_removal_tests;
