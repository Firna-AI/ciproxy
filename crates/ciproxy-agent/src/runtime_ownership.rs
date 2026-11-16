//! The existing runtime lock is independent of workload mounts and recovery.

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::sync::Arc;

use fs2::FileExt;

use crate::error::{ErrorResultExt, Result};

/// Holding the last reference keeps one existing runtime lock claimed.
#[cfg_attr(test, unimock::unimock(api = RuntimeOwnerMock))]
pub trait RuntimeOwner: Send + Sync {}

impl RuntimeOwner for File {}

/// Claim the existing runtime exclusion without touching workload directories.
#[cfg_attr(test, unimock::unimock(api = RuntimeClaimsMock))]
pub(crate) trait RuntimeClaims: Send + Sync {
    fn claim(&self) -> Result<Arc<dyn RuntimeOwner>>;
}

pub(crate) struct LinuxRuntimeClaims {
    pub(crate) control: PathBuf,
}

impl RuntimeClaims for LinuxRuntimeClaims {
    fn claim(&self) -> Result<Arc<dyn RuntimeOwner>> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(self.control.join("images.lock"))
            .context("open image ownership lock")?;
        FileExt::try_lock_exclusive(&file).context("claim image cache ownership")?;
        Ok(Arc::new(file))
    }
}
