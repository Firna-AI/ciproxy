//! Reserve shared resources and hold the selected image through VM preparation.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::reports::DrainState;
use ciproxy_protocol::runners::{RunnerReport, RunnerSpec, RunnerState};
use ciproxy_protocol::version::ProtocolVersion;

use crate::capacity::machine_resources;
use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::slot_allocation::choose;
use crate::state::{Mode, VmRecord};

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
            let mut state = self.state.lock().await;
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
            let mut vm = self.vms.allocate(
                RunnerReport {
                    configuration_lost: false,
                    slot_indices: slot_indices.clone(),
                    runner_id: runner.runner_id.clone(),
                    organization_id: runner.organization_id.clone(),
                    label: runner.label.clone(),
                    requested_size: runner.size,
                    size,
                    image_id: metadata.image_id,
                    state: RunnerState::Starting,
                },
                &state.runners,
            )?;
            vm.report.slot_indices = slot_indices;
            let mut next = state.clone();
            next.runners.push(vm.clone());
            self.store.save(&next)?;
            *state = next;
            (vm, available.available([size]).job_disk_bytes, image)
        };
        let cache_budget = self.cache_budget(cache_budget)?;
        if let Err(error) = self.cache.budget(cache_budget).await {
            tracing::warn!(error = %error, "cache space is not released; preserve VM disk admission");
            self.retire(&vm).await?;
            return Err(error);
        }
        self.launch(vm, configure(runner)).await
    }

    async fn retry_start(&self, vm: VmRecord, runner: RunnerSpec) -> Result<()> {
        if vm.report.organization_id != runner.organization_id
            || vm.report.label != runner.label
            || vm.report.requested_size != runner.size
            || runner.slot_indices != vm.report.slot_indices
        {
            return Err(Error::RunnerMismatch);
        }
        match vm.report.state {
            RunnerState::Starting => {
                let message = configure(runner);
                self.record_configuration(&vm.report.runner_id.0, false)
                    .await?;
                self.vms.send(&vm, &message).await
            }
            RunnerState::Warm | RunnerState::Running { .. } => Ok(()),
            RunnerState::Stopping => Err(Error::RunnerMismatch),
        }
    }

    async fn launch(&self, vm: VmRecord, config: AgentToGuest) -> Result<()> {
        match self.vms.start(&vm, config).await {
            Ok(()) => Ok(()),
            Err(error) => {
                if self.vms.stop(&vm).await.is_ok() {
                    let mut state = self.state.lock().await;
                    let mut next = state.clone();
                    next.runners.retain(|record| record.local_id != vm.local_id);
                    self.store.save(&next)?;
                    *state = next;
                }
                Err(error)
            }
        }
    }
}

fn configure(runner: RunnerSpec) -> AgentToGuest {
    AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: runner.runner_id,
        organization_id: runner.organization_id,
        registration: runner.registration,
        cache: None,
    }
}
