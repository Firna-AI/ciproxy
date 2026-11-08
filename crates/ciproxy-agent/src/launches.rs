//! Reserve shared resources and hold the selected image through VM preparation.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::reports::DrainState;
use ciproxy_protocol::runners::{RunnerReport, RunnerSpec, RunnerState};

use crate::capacity::machine_resources;
use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::registration::{RegistrationAction, configure};
use crate::slot_allocation::choose;
use crate::state::{Mode, RegistrationClaim, VmRecord};

impl Controller {
    pub(crate) async fn start(&self, runner: RunnerSpec) -> Result<()> {
        let report = self.machine_report().await?;
        if report.drain != DrainState::Active {
            return Err(Error::Draining);
        }
        let existing = self
            .state
            .lock()
            .await
            .runners
            .iter()
            .find(|vm| vm.report.runner_id == runner.runner_id)
            .cloned();
        if let Some(vm) = existing {
            return self.retry_start(vm, runner).await;
        }
        let (vm, cache_budget, _image) = {
            let _allocation = self.allocation.lock().await;
            let (report, used, cache_budget, image) = {
                let state = self.state.lock().await;
                if state.mode != Mode::Active {
                    return Err(Error::Draining);
                }
                if state
                    .runners
                    .iter()
                    .any(|vm| vm.report.runner_id == runner.runner_id)
                {
                    return Err(Error::RunnerMismatch);
                }
                let image = self.images.hold_newest()?;
                let metadata = image.metadata();
                let size = RootSizing::for_image(&metadata)
                    .vm_size(runner.size)
                    .ok_or(Error::Capacity)?;
                let available = machine_resources(&report.hardware, self.image_budget)
                    .available(state.runners.iter().map(|vm| vm.report.size));
                let leased = self
                    .dockers
                    .as_ref()
                    .map(|dockers| dockers.report().map(|report| report.leased_bytes))
                    .transpose()?
                    .unwrap_or(0);
                if available.job_disk_bytes.saturating_sub(leased) < size.disk_bytes
                    || !available.fits(size)
                {
                    return Err(Error::Capacity);
                }
                let slot_indices = choose(
                    size,
                    &runner.slot_indices,
                    report.capacity.0,
                    &state.runners,
                )?;
                let report = RunnerReport {
                    configuration_lost: false,
                    slot_indices: slot_indices.clone(),
                    runner_id: runner.runner_id.clone(),
                    organization_id: runner.organization_id.clone(),
                    label: runner.label.clone(),
                    requested_size: runner.size,
                    size,
                    image_id: metadata.image_id,
                    state: RunnerState::Starting,
                };
                let used = state
                    .runners
                    .iter()
                    .map(|vm| vm.network_index)
                    .chain(state.pending_networks.iter().copied())
                    .collect::<Vec<_>>();
                (
                    report,
                    used,
                    available.available([size]).job_disk_bytes,
                    image,
                )
            };
            let slot_indices = report.slot_indices.clone();
            let mut vm = self.vms.allocate(report, &used).await?;
            vm.report.slot_indices = slot_indices;
            let mut state = self.state.lock().await;
            if state.mode != Mode::Active {
                return Err(Error::Draining);
            }
            let mut next = state.clone();
            next.runners.push(vm.clone());
            self.store.save(&next)?;
            *state = next;
            (vm, cache_budget, image)
        };
        let cache_budget = self.cache_budget(cache_budget)?;
        if let Err(error) = self.cache.budget(cache_budget).await {
            tracing::warn!(error = %error, "cache space is not released; preserve VM disk admission");
            self.retire(&vm).await?;
            return Err(error);
        }
        self.launch(vm, configure(runner)).await
    }

    async fn launch(&self, vm: VmRecord, config: AgentToGuest) -> Result<()> {
        let vm = match self
            .claim_registration(&vm, RegistrationClaim::Delivery)
            .await?
        {
            RegistrationAction::Claimed(vm) => vm,
            RegistrationAction::Preserve => return Ok(()),
            RegistrationAction::GuestShutdown(_) => return Err(Error::RunnerMismatch),
        };
        let result = self.vms.start(&vm, config).await;
        self.release_registration(&vm, RegistrationClaim::Delivery)
            .await;
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                if let Err(cleanup_error) = self.vms.stop(&vm).await {
                    tracing::warn!(%cleanup_error, vm_id = %vm.local_id, "failed-start VM cleanup remains owned");
                    if let Err(record_error) = self.mark_failed_cleanup(&vm).await {
                        tracing::warn!(%record_error, "failed-start cleanup intent could not be saved");
                    }
                } else if let Err(cleanup_error) = self.finish_vm_cleanup(&vm).await {
                    tracing::warn!(%cleanup_error, "failed-start network ownership could not be saved");
                }
                Err(error)
            }
        }
    }
}

#[cfg(test)]
#[path = "_tests_/registration_claim_tests.rs"]
mod registration_claim_tests;
