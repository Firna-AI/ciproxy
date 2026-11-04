//! Bind vsock observations to their VM and persist every job transition.

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::version::ProtocolVersion;

use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::{Mode, VmRecord};

impl Controller {
    pub(crate) async fn observe(
        &self,
        runner_id: &str,
        message: GuestToAgent,
    ) -> Result<Option<AgentMessage>> {
        let vm = self
            .state
            .lock()
            .await
            .runners
            .iter()
            .find(|vm| vm.report.runner_id.0 == runner_id)
            .cloned()
            .ok_or(Error::GuestMismatch)?;
        let response = match message {
            GuestToAgent::Hello {
                protocol_version,
                image_id,
            } => {
                if protocol_version != ProtocolVersion::CURRENT || image_id != vm.report.image_id {
                    return Err(Error::GuestMismatch);
                }
                if self.state.lock().await.mode != Mode::Active
                    || vm.report.state == RunnerState::Stopping
                {
                    self.request_idle_stop(&vm).await?;
                }
                return Ok(None);
            }
            GuestToAgent::RunnerReady { runner_id } => {
                if runner_id != vm.report.runner_id {
                    return Err(Error::GuestMismatch);
                }
                let mut state = self.state.lock().await;
                if state.mode != Mode::Active {
                    return Ok(None);
                }
                let mut next = state.clone();
                let record = next
                    .runners
                    .iter_mut()
                    .find(|r| r.local_id == vm.local_id)
                    .ok_or(Error::GuestMismatch)?;
                if matches!(
                    record.report.state,
                    RunnerState::Starting | RunnerState::Warm
                ) {
                    record.report.state = RunnerState::Warm;
                }
                self.store.save(&next)?;
                *state = next;
                AgentMessage::RunnerReady { runner_id }
            }
            GuestToAgent::JobStarted { runner_id, job_id } => {
                if runner_id != vm.report.runner_id {
                    return Err(Error::GuestMismatch);
                }
                let mut state = self.state.lock().await;
                let mut next = state.clone();
                let record = next
                    .runners
                    .iter_mut()
                    .find(|r| r.local_id == vm.local_id)
                    .ok_or(Error::GuestMismatch)?;
                if let RunnerState::Running { job_id: previous } = &record.report.state
                    && *previous != job_id
                {
                    return Err(Error::GuestMismatch);
                }
                record.report.state = RunnerState::Running {
                    job_id: job_id.clone(),
                };
                self.store.save(&next)?;
                *state = next;
                tracing::info!(event = "job_started_observed", vm_id = %vm.local_id, "accepted guest job state");
                AgentMessage::JobStarted { runner_id, job_id }
            }
            GuestToAgent::RunnerExited { runner_id, exit } => {
                if runner_id != vm.report.runner_id {
                    return Err(Error::GuestMismatch);
                }
                self.retire(&vm).await?;
                AgentMessage::RunnerExited { runner_id, exit }
            }
        };
        Ok(Some(response))
    }

    pub(crate) async fn request_idle_stop(&self, vm: &VmRecord) -> Result<()> {
        if matches!(vm.report.state, RunnerState::Running { .. }) {
            return Ok(());
        }
        self.vms.request_stop(vm).await?;
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        if let Some(record) = next
            .runners
            .iter_mut()
            .find(|record| record.local_id == vm.local_id)
            && !matches!(
                record.report.state,
                RunnerState::Running { .. } | RunnerState::Stopping
            )
        {
            record.report.state = RunnerState::Stopping;
            self.store.save(&next)?;
            *state = next;
        }
        Ok(())
    }

    pub(crate) async fn retire(&self, vm: &VmRecord) -> Result<()> {
        self.vms.stop(vm).await?;
        if let Err(error) = self.cache.release(&vm.report.runner_id).await {
            tracing::warn!(error = %error, "machine cache release failed");
        }
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        next.runners.retain(|record| record.local_id != vm.local_id);
        self.store.save(&next)?;
        *state = next;
        Ok(())
    }
}
