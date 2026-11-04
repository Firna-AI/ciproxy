//! Reserve VM resources durably before starting a separate unit.

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::reports::DrainState;
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::runners::{RunnerReport, RunnerSpec, RunnerState};
use ciproxy_protocol::version::ProtocolVersion;

use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::{Mode, VmRecord};

impl Controller {
    pub(crate) async fn start(&self, runner: RunnerSpec) -> Result<()> {
        let report = self.machine_report().await?;
        if report.drain != DrainState::Active {
            return Err(Error::Draining);
        }
        let image_id = report.images.newest.ok_or(Error::NoImage)?;
        let (vm, cache_budget) = {
            let mut state = self.state.lock().await;
            if state.mode != Mode::Active {
                return Err(Error::Draining);
            }
            if state
                .runners
                .iter()
                .any(|vm| vm.report.runner_id == runner.runner_id)
            {
                return Ok(());
            }
            let used = state
                .runners
                .iter()
                .map(|vm| Slots::for_vm(vm.report.size).0)
                .sum::<u64>();
            let disk_used = state
                .runners
                .iter()
                .map(|vm| vm.report.size.disk_bytes)
                .sum::<u64>();
            let disk_fits = report.hardware.disks.first().is_some_and(|disk| {
                disk.capacity_bytes.saturating_mul(3) / 4
                    >= disk_used.saturating_add(runner.size.disk_bytes)
            });
            if report.capacity.0.saturating_sub(used) < Slots::for_vm(runner.size).0 || !disk_fits {
                return Err(Error::Capacity);
            }
            let vm = self.vms.allocate(
                RunnerReport {
                    runner_id: runner.runner_id.clone(),
                    organization_id: runner.organization_id.clone(),
                    label: runner.label,
                    size: runner.size,
                    image_id,
                    state: RunnerState::Starting,
                },
                &state.runners,
            )?;
            let mut next = state.clone();
            next.runners.push(vm.clone());
            self.store.save(&next)?;
            *state = next;
            let budget = report.hardware.disks.first().map_or(0, |disk| {
                (disk.capacity_bytes.saturating_mul(3) / 4)
                    .saturating_sub(disk_used.saturating_add(runner.size.disk_bytes))
            });
            (vm, budget)
        };
        let config = AgentToGuest::ConfigureRunner {
            protocol_version: ProtocolVersion::CURRENT,
            runner_id: runner.runner_id,
            organization_id: runner.organization_id,
            registration: runner.registration,
            cache: None,
        };
        if let Err(error) = self.cache.budget(cache_budget).await {
            tracing::warn!(error = %error, "cache reclaim unavailable; continue with GitHub caching");
        }
        self.launch(vm, config).await
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
