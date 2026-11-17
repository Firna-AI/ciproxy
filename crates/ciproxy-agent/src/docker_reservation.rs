//! Save each Docker grant and release its backing space under admission's lock.

use crate::docker::LinuxDockers;
use crate::docker_files::{self, Lease};
use crate::error::{Error, Result};
use crate::state::VmRecord;

impl LinuxDockers {
    pub(crate) async fn reserve(&self, vm: &VmRecord, lease: &Lease, spare: u64) -> Result<()> {
        let _allocation = self.allocation.lock().await;
        let free_pool = self.space.available(self.image_budget).await?;
        let previous = docker_files::load(&self.layout, vm)?.map_or(0, |held| held.bytes);
        let other = docker_files::held(&self.layout)?.saturating_sub(previous);
        if lease.bytes > spare.min(free_pool).saturating_sub(other) {
            return Err(Error::Capacity);
        }
        docker_files::save(&self.layout, vm, lease)?;
        self.space.release(free_pool, Some(self)).await
    }
}
