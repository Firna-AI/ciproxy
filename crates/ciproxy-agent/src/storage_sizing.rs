//! Reserve images first. Keep XFS overhead in the remaining host quarter.
//!
//! The image budget setting is shared: macOS reads it too, but has no XFS
//! area, so only the setting applies there.

use ciproxy_protocol::resources::{DEFAULT_IMAGE_BUDGET_BYTES, Resources};

use crate::error::{Result, repair};

pub(crate) const DEFAULT_IMAGE_BYTES: u64 = DEFAULT_IMAGE_BUDGET_BYTES;
pub(crate) const MINIMUM_DISK_BYTES: u64 = 250_000_000_000;
pub(crate) const SUPPORT_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const BLOCK: u64 = 4096;
/// The image budget setting of Linux and macOS.
pub(crate) const IMAGE_BUDGET_SETTING: &str = "CIPROXY_IMAGE_BUDGET_BYTES";

#[derive(Clone, Copy, Debug)]
pub(crate) struct AreaSizing {
    pub(crate) host: u64,
    pub(crate) images: u64,
}

impl AreaSizing {
    pub(crate) fn job(self) -> u64 {
        Resources::for_machine_with_image_budget(0, 0, self.host, self.images, false).job_disk_bytes
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

/// Read `CIPROXY_IMAGE_BUDGET_BYTES` from the environment. `None` means that
/// the setting is absent, so the platform default applies.
pub(crate) fn requested_image_budget() -> Result<Option<u64>> {
    match std::env::var(IMAGE_BUDGET_SETTING) {
        Ok(value) => setting(Some(&value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(repair(
            "image budget setting",
            "Set CIPROXY_IMAGE_BUDGET_BYTES to an integer byte count.",
        )),
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
