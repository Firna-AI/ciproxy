//! Full hardware and slot reports, including every live VM after reconnect.

use ciproxy_protocol::reports::{DrainState, MachineReport, SlotOccupancy, SlotReport};
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::statistics::Statistics;

use crate::capacity::machine_resources;
use crate::controller::Controller;
use crate::error::Result;
use crate::slot_allocation::allocations;
use crate::state::Mode;

impl Controller {
    pub(crate) async fn machine_report(&self) -> Result<MachineReport> {
        let hardware = self.host.hardware()?;
        let capacity = Slots::for_machine(
            hardware.threads,
            hardware.memory_bytes,
            hardware.virtualization.is_available(),
        );
        let job_disk_bytes = machine_resources(&hardware, self.image_budget).job_disk_bytes;
        let cache = match self.cache.report().await {
            Ok(report) => Statistics::Measured(report),
            Err(error) => {
                tracing::warn!(error = %error, "machine cache reports unavailable");
                Statistics::Unavailable
            }
        };
        let docker_disks = match &self.dockers {
            Some(dockers) => match dockers.report() {
                Ok(report) => Statistics::Measured(report),
                Err(error) => {
                    tracing::warn!(%error, "Docker disk reports unavailable");
                    Statistics::Unavailable
                }
            },
            None => Statistics::Unavailable,
        };
        let images = self.images.report()?;
        let state = self.state.lock().await;
        let runners = state
            .runners
            .iter()
            .map(|vm| vm.report.clone())
            .collect::<Vec<_>>();
        let occupied = allocations(&state.runners);
        let slots = (0..capacity.0)
            .map(|index| {
                let runner = state
                    .runners
                    .iter()
                    .zip(&occupied)
                    .find(|(_, slots)| slots.contains(&index))
                    .map(|(vm, _)| &vm.report);
                let occupancy = match runner {
                    Some(runner) => match &runner.state {
                        RunnerState::Running { job_id } => SlotOccupancy::Job {
                            runner_id: runner.runner_id.clone(),
                            job_id: job_id.clone(),
                        },
                        _ => SlotOccupancy::Warm {
                            runner_id: runner.runner_id.clone(),
                        },
                    },
                    None => SlotOccupancy::Free,
                };
                SlotReport { index, occupancy }
            })
            .collect();
        Ok(MachineReport {
            hostname: self.host.hostname()?,
            agent_version: env!("CARGO_PKG_VERSION").to_owned(),
            hardware,
            capacity,
            job_disk_bytes,
            slots,
            runners,
            drain: if state.mode == Mode::Active {
                DrainState::Active
            } else {
                DrainState::Draining
            },
            cache,
            docker_disks,
            images,
        })
    }
}

impl Controller {
    pub(crate) async fn initialize_report_budget(&self) -> Result<()> {
        let _allocation = self.allocation.lock().await;
        let free = self.space.available(self.image_budget).await?;
        let budget = match &self.dockers {
            Some(dockers) => free
                .saturating_sub(dockers.report()?.leased_bytes)
                .saturating_sub(dockers.unused_bytes()?),
            None => free,
        };
        self.cache.refresh_budget(budget).await
    }
}

#[cfg(test)]
#[path = "_tests_/reporting_tests.rs"]
mod reporting_tests;
