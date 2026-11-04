//! Full hardware and slot reports, including every live VM after reconnect.

use ciproxy_protocol::cache::{DockerDiskReport, MachineCacheReport};
use ciproxy_protocol::reports::{DrainState, MachineReport, SlotOccupancy, SlotReport};
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::runners::RunnerState;

use crate::controller::Controller;
use crate::error::Result;
use crate::state::Mode;

impl Controller {
    pub(crate) async fn machine_report(&self) -> Result<MachineReport> {
        let hardware = self.host.hardware()?;
        let capacity = Slots::for_machine(hardware.threads, hardware.memory_bytes, hardware.kvm);
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
        let mut occupants = runners.iter().flat_map(|runner| {
            let occupancy = match &runner.state {
                RunnerState::Running { job_id } => SlotOccupancy::Job {
                    runner_id: runner.runner_id.clone(),
                    job_id: job_id.clone(),
                },
                _ => SlotOccupancy::Warm {
                    runner_id: runner.runner_id.clone(),
                },
            };
            std::iter::repeat_n(occupancy, Slots::for_vm(runner.size).0 as usize)
        });
        let slots = (0..capacity.0)
            .map(|index| SlotReport {
                index,
                occupancy: occupants.next().unwrap_or(SlotOccupancy::Free),
            })
            .collect();
        Ok(MachineReport {
            hostname: self.host.hostname()?,
            agent_version: env!("CARGO_PKG_VERSION").to_owned(),
            hardware,
            capacity,
            slots,
            runners,
            drain: if state.mode == Mode::Active {
                DrainState::Active
            } else {
                DrainState::Draining
            },
            cache,
            docker_disks: DockerDiskReport {
                repositories: 0,
                used_bytes: 0,
            },
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
        jobs: Vec::new(),
    }
}
