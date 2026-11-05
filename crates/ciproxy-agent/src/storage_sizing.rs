//! Keep the complete protocol job disk. Images and XFS costs use the host quarter.

use ciproxy_protocol::resources::Resources;

use crate::error::Result;
use crate::host_system::repair;

pub(crate) const DEFAULT_IMAGE_BYTES: u64 = 4_000_000_000;
pub(crate) const SUPPORT_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const BLOCK: u64 = 4096;

#[derive(Clone, Copy, Debug)]
pub(crate) struct AreaSizing {
    pub(crate) host: u64,
    pub(crate) images: u64,
}

impl AreaSizing {
    pub(crate) fn job(self) -> u64 {
        Resources::for_machine(0, 0, self.host, false).job_disk_bytes
    }

    pub(crate) fn length(self, overhead: u64) -> Result<u64> {
        let length = self
            .job()
            .checked_add(self.images)
            .and_then(|bytes| bytes.checked_add(overhead))
            .and_then(|bytes| bytes.checked_add(BLOCK - 1))
            .map(|bytes| bytes / BLOCK * BLOCK);
        match length {
            Some(length) if self.images > SUPPORT_BYTES && length < self.host => Ok(length),
            _ => Err(repair(
                "image budget",
                "Set a smaller CIPROXY_IMAGE_BUDGET_BYTES value. Keep system space in the host quarter.",
            )),
        }
    }

    pub(crate) fn measured(self, backing_bytes: u64, available: u64) -> Result<u64> {
        self.length(
            backing_bytes
                .saturating_sub(available)
                .saturating_add(self.metadata_safety()),
        )
    }

    pub(crate) fn metadata_safety(self) -> u64 {
        self.job() / 32 + SUPPORT_BYTES
    }
}

pub(crate) fn setting(value: Option<&str>) -> Result<Option<u64>> {
    match value {
        None => Ok(None),
        Some(value) => match value.parse::<u64>() {
            Ok(bytes) if bytes > SUPPORT_BYTES => Ok(Some(bytes)),
            _ => Err(repair(
                "image budget setting",
                "Set CIPROXY_IMAGE_BUDGET_BYTES to an integer greater than 16777216.",
            )),
        },
    }
}

#[cfg(test)]
#[path = "_tests_/storage_sizing_tests.rs"]
mod storage_sizing_tests;
