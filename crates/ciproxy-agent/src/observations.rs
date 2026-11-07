//! Bind vsock observations to their VM and persist every job transition.

use std::sync::atomic::Ordering;

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::version::ProtocolVersion;

use crate::boundaries::GuestProvenance;
use crate::cache_authority::CacheAuthority;
use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::{Mode, VmRecord};

impl Controller {
    pub(crate) async fn record_configuration(&self, runner_id: &str, lost: bool) -> Result<()> {
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        if let Some(vm) = next
            .runners
            .iter_mut()
            .find(|vm| vm.report.runner_id.0 == runner_id)
            && vm.report.configuration_lost != lost
            && (!lost || vm.report.state == RunnerState::Starting)
        {
            vm.report.configuration_lost = lost;
            self.store.save(&next)?;
            *state = next;
        }
        Ok(())
    }
    pub(crate) async fn observe(
        &self,
        provenance: GuestProvenance,
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
            GuestToAgent::JobStarting {
                runner_id,
                job_id,
                runtime,
            } => {
                if runner_id != vm.report.runner_id {
                    return Err(Error::GuestMismatch);
                }
                let (updated, checked) = self
                    .started_record(
                        &vm,
                        &job_id,
                        Some(runtime),
                        provenance == GuestProvenance::InitialConfiguration,
                    )
                    .await?;
                if let Some(runtime) = checked {
                    if let Err(error) = self.cache.handoff(&updated, &job_id, runtime).await {
                        tracing::warn!(%error,"cache assignment unavailable; preserve GitHub fallback");
                    }
                    if let Some(facts) = updated.job_facts.clone() {
                        self.prepare_job(&updated, facts).await?;
                    }
                } else {
                    tracing::warn!("cache assignment authority unavailable; select GitHub caching");
                    self.release_github(&updated).await?;
                }
                if self.state.lock().await.mode != Mode::Active
                    || !self.control_connected.load(Ordering::Acquire)
                {
                    self.release_pending_jobs().await?;
                }
                AgentMessage::JobStarted { runner_id, job_id }
            }
            GuestToAgent::JobPrepared {
                runner_id,
                job_id,
                outcome,
            } => {
                if runner_id != vm.report.runner_id
                    || vm.report.state != (RunnerState::Running { job_id })
                {
                    return Err(Error::GuestMismatch);
                }
                if !outcome.cache_ready
                    && let Err(error) = self.cache.fallback(&vm).await
                {
                    tracing::warn!(%error,"cache fallback cleanup will retry");
                }
                if !outcome.docker_attached
                    && let Some(dockers) = &self.dockers
                    && dockers.reserved(&vm)?
                    && dockers.failed(&vm)?
                {
                    self.vms.clear_docker(&vm).await?;
                    dockers.discard(&vm).await?;
                }
                self.prepared_record(&vm, outcome).await?;
                if !outcome.cache_ready && vm.cache_authority != CacheAuthority::GitHubOnly {
                    self.release_github(&vm).await?;
                }
                tracing::info!(event="job_prepared_observed",vm_id=%vm.local_id,"accepted local job readiness");
                return Ok(None);
            }
            GuestToAgent::Hello {
                protocol_version,
                image_id,
                configuration_received,
                ..
            } => {
                if protocol_version != ProtocolVersion::CURRENT || image_id != vm.report.image_id {
                    return Err(Error::GuestMismatch);
                }
                if configuration_received {
                    self.record_configuration(runner_id, false).await?;
                }
                if self.state.lock().await.mode == Mode::Removed
                    || vm.report.state == RunnerState::Stopping
                {
                    self.request_idle_stop(&vm).await?;
                }
                return Ok(None);
            }
            GuestToAgent::RunnerConfigured {
                runner_id: received,
            } => {
                if received != vm.report.runner_id {
                    return Err(Error::GuestMismatch);
                }
                self.record_configuration(runner_id, false).await?;
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
                self.started_record(&vm, &job_id, None, false).await?;
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
        if let Some(dockers) = &self.dockers {
            if dockers.pending(vm)? {
                dockers.discard(vm).await?;
            }
            dockers.release(vm)?;
        }
        self.cache.release(vm).await?;
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        next.runners.retain(|record| record.local_id != vm.local_id);
        self.store.save(&next)?;
        *state = next;
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/preparation_outcome_tests.rs"]
mod preparation_outcome_tests;

#[cfg(test)]
#[path = "_tests_/preparation_replay_tests.rs"]
mod preparation_replay_tests;

#[cfg(test)]
#[path = "_tests_/docker_recovery_tests.rs"]
mod docker_recovery_tests;
