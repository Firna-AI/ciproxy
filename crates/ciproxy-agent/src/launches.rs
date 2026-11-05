//! Reserve shared resources and hold the selected image through VM preparation.

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::reports::DrainState;
use ciproxy_protocol::runners::{RunnerReport, RunnerSpec, RunnerState};
use ciproxy_protocol::version::ProtocolVersion;

use crate::capacity::machine_resources;
use crate::controller::Controller;
use crate::error::{Error, Result};
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
            let available = machine_resources(&report.hardware)
                .available(state.runners.iter().map(|vm| vm.report.size));
            if !available.fits(runner.size) {
                return Err(Error::Capacity);
            }
            let image = self.images.hold_newest()?;
            let vm = self.vms.allocate(
                RunnerReport {
                    runner_id: runner.runner_id.clone(),
                    organization_id: runner.organization_id.clone(),
                    label: runner.label.clone(),
                    size: runner.size,
                    image_id: image.image_id(),
                    state: RunnerState::Starting,
                },
                &state.runners,
            )?;
            let mut next = state.clone();
            next.runners.push(vm.clone());
            self.store.save(&next)?;
            *state = next;
            (vm, available.available([runner.size]).job_disk_bytes, image)
        };
        if let Err(error) = self.cache.budget(cache_budget).await {
            tracing::warn!(error = %error, "cache reclaim unavailable; continue with GitHub caching");
        }
        self.launch(vm, configure(runner)).await
    }

    async fn retry_start(&self, vm: VmRecord, runner: RunnerSpec) -> Result<()> {
        if vm.report.organization_id != runner.organization_id
            || vm.report.label != runner.label
            || vm.report.size != runner.size
        {
            return Err(Error::RunnerMismatch);
        }
        match vm.report.state {
            RunnerState::Starting => {
                let mut message = configure(runner);
                if let AgentToGuest::ConfigureRunner { cache, .. } = &mut message {
                    *cache = self.cache.configure(&vm).await.unwrap_or_else(|error| {
                        tracing::warn!(error = %error, "cache unavailable during registration replay"); None
                    });
                }
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
