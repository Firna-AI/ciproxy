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
    pub(crate) fn cache_budget(&self, free_pool: u64) -> Result<u64> {
        if let Some(dockers) = &self.dockers {
            let free = free_pool.saturating_sub(dockers.report()?.leased_bytes);
            Ok(free.saturating_sub(dockers.reclaim(free)?))
        } else {
            Ok(free_pool)
        }
    }
}
