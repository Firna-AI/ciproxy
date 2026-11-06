//! Full hardware and slot reports, including every live VM after reconnect.

use ciproxy_protocol::cache::{DockerDiskReport, MachineCacheReport};
use ciproxy_protocol::reports::{DrainState, MachineReport, SlotOccupancy, SlotReport};
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::runners::RunnerState;

use crate::capacity::machine_resources;
use crate::controller::Controller;
use crate::error::Result;
use crate::slot_allocation::allocations;
use crate::state::Mode;

impl Controller {
    pub(crate) async fn machine_report(&self) -> Result<MachineReport> {
        let hardware = self.host.hardware()?;
        let capacity = Slots::for_machine(hardware.threads, hardware.memory_bytes, hardware.kvm);
        let job_disk_bytes = machine_resources(&hardware, self.image_budget).job_disk_bytes;
        let cache = self.cache.report().await.unwrap_or_else(|error| {
            tracing::warn!(error = %error, "machine cache reports unavailable");
            empty_cache()
        });
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
            docker_disks: self
                .dockers
                .as_ref()
                .and_then(|dockers| dockers.report().ok())
                .unwrap_or(DockerDiskReport {
                    leased_bytes: 0,
                    repositories: 0,
                    used_bytes: 0,
                }),
            images,
        })
    }
}

pub(crate) fn empty_cache() -> MachineCacheReport {
    MachineCacheReport {
        used_bytes: 0,
        capacity_bytes: 0,
        restores_last_7_days: 0,
        hits_last_7_days: 0,
        unknown_jobs: Vec::new(),
        jobs: Vec::new(),
    }
}
