//! Select the reported job disk and delegate every fit rule to the protocol.

use crate::controller::Controller;
use crate::error::Result;
use ciproxy_protocol::reports::HardwareReport;
use ciproxy_protocol::resources::Resources;

pub(crate) fn machine_resources(hardware: &HardwareReport, image_budget: u64) -> Resources {
    Resources::for_machine_with_image_budget(
        hardware.threads,
        hardware.memory_bytes,
        hardware.disks.first().map_or(0, |disk| disk.capacity_bytes),
        image_budget,
        hardware.kvm,
    )
}

impl Controller {
    pub(crate) async fn initialize_report_budget(&self) -> Result<()> {
        let _allocation = self.allocation.lock().await;
        let hardware = self.host.hardware()?;
        let free = {
            let state = self.state.lock().await;
            machine_resources(&hardware, self.image_budget)
                .available(state.runners.iter().map(|vm| vm.report.size))
                .job_disk_bytes
        };
        let budget = match &self.dockers {
            Some(dockers) => free
                .saturating_sub(dockers.report()?.leased_bytes)
                .saturating_sub(dockers.unused_bytes()?),
            None => free,
        };
        self.cache.refresh_budget(budget).await
    }

    pub(crate) fn cache_budget(&self, free_pool: u64) -> Result<u64> {
        if let Some(dockers) = &self.dockers {
            let free = free_pool.saturating_sub(dockers.report()?.leased_bytes);
            Ok(free.saturating_sub(dockers.reclaim(free)?))
        } else {
            Ok(free_pool)
        }
    }
}
