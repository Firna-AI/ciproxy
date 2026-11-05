//! Physical image allocation is reserved before a file can consume workload space.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::error::{Error, ErrorResultExt, Result};
use crate::storage_sizing::SUPPORT_BYTES;

#[cfg_attr(test, unimock::unimock(api = SpaceUsageMock))]
pub(crate) trait SpaceUsage: Send + Sync {
    fn used(&self) -> Result<u64>;
}

#[cfg_attr(test, unimock::unimock(api = SpaceClaimMock))]
/// A reservation remains charged until its last owner releases it.
pub trait SpaceClaim: Send + Sync {}

#[cfg_attr(test, unimock::unimock(api = ImageSpaceMock))]
pub(crate) trait ImageSpace: Send + Sync {
    fn claim(&self, bytes: u64) -> Result<Arc<dyn SpaceClaim>>;
}

struct Ledger {
    limit: u64,
    reserved: u64,
}

pub(crate) struct ImageBudget {
    usage: Arc<dyn SpaceUsage>,
    ledger: Arc<Mutex<Ledger>>,
}

impl ImageBudget {
    pub(crate) fn new(limit: u64, usage: Arc<dyn SpaceUsage>) -> Self {
        Self {
            usage,
            ledger: Arc::new(Mutex::new(Ledger { limit, reserved: 0 })),
        }
    }
}

impl ImageSpace for ImageBudget {
    fn claim(&self, bytes: u64) -> Result<Arc<dyn SpaceClaim>> {
        let mut ledger = self
            .ledger
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let used = self.usage.used()?;
        let remaining = ledger
            .limit
            .saturating_sub(used)
            .saturating_sub(ledger.reserved);
        if used > ledger.limit || bytes > remaining {
            return Err(Error::ImageBudget);
        }
        ledger.reserved += bytes;
        Ok(Arc::new(Reservation {
            bytes,
            ledger: self.ledger.clone(),
        }))
    }
}

struct Reservation {
    bytes: u64,
    ledger: Arc<Mutex<Ledger>>,
}
impl SpaceClaim for Reservation {}
impl Drop for Reservation {
    fn drop(&mut self) {
        let mut ledger = self
            .ledger
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        ledger.reserved = ledger.reserved.saturating_sub(self.bytes);
    }
}

pub(crate) struct LinuxSpaceUsage {
    pub(crate) area: PathBuf,
}
pub(crate) const VM_SUPPORT_BYTES: u64 = 8 * 1024 * 1024;

impl SpaceUsage for LinuxSpaceUsage {
    fn used(&self) -> Result<u64> {
        Ok(SUPPORT_BYTES
            .saturating_add(allocated(&self.area.join("images"), None)?)
            .saturating_add(allocated(
                &self.area.join("jailer"),
                Some(&self.area.join("jailer")),
            )?))
    }
}

fn allocated(path: &Path, jail: Option<&Path>) -> Result<u64> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error).context("measure image allocation"),
    };
    if metadata.is_dir() {
        let mut bytes = metadata
            .blocks()
            .saturating_mul(512)
            .saturating_add(16 * 1024);
        if jail.is_some_and(|base| jail_root(path, base)) {
            bytes = bytes.saturating_add(VM_SUPPORT_BYTES);
        }
        for entry in fs::read_dir(path).context("list image allocation")? {
            bytes = bytes.saturating_add(allocated(
                &entry.context("read image allocation entry")?.path(),
                jail,
            )?);
        }
        return Ok(bytes);
    }
    if jail.is_some_and(|base| path.parent().is_some_and(|parent| jail_root(parent, base)))
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, "rootfs.ext4" | "docker.ext4"))
    {
        return Ok(0);
    }
    Ok(metadata
        .blocks()
        .saturating_mul(512)
        .saturating_add(16 * 1024))
}

fn jail_root(path: &Path, base: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(base) else {
        return false;
    };
    let parts = relative
        .iter()
        .filter_map(|part| part.to_str())
        .collect::<Vec<_>>();
    matches!(parts.as_slice(), ["firecracker", id, "root"] if id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
#[path = "_tests_/image_space_tests.rs"]
mod image_space_tests;
