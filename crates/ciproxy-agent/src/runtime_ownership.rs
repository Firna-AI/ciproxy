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

struct LockedRuntime {
    file: File,
}

impl RuntimeOwner for LockedRuntime {}

/// Release ownership even while a spawned tool briefly inherits the descriptor.
impl Drop for LockedRuntime {
    fn drop(&mut self) {
        if let Err(error) = FileExt::unlock(&self.file) {
            tracing::warn!(%error, "runtime ownership lock could not be released");
        }
    }
}

/// Claim the existing runtime exclusion without touching workload directories.
#[cfg_attr(test, unimock::unimock(api = RuntimeClaimsMock))]
pub(crate) trait RuntimeClaims: Send + Sync {
    fn claim(&self) -> Result<Arc<dyn RuntimeOwner>>;
    fn wait(&self) -> Result<Arc<dyn RuntimeOwner>>;
}

/// Report once when an uninstall must wait for another runtime owner.
#[cfg_attr(test, unimock::unimock(api = RuntimeWaitNoticeMock))]
pub(crate) trait RuntimeWaitNotice: Send + Sync {
    fn waiting(&self);
}

pub(crate) struct CliRuntimeWaitNotice;

impl RuntimeWaitNotice for CliRuntimeWaitNotice {
    fn waiting(&self) {
        println!("Another ciproxy-agent process is running. Stop it to continue uninstall.");
    }
}

pub(crate) struct FileRuntimeClaims {
    pub(crate) control: PathBuf,
    pub(crate) notice: Arc<dyn RuntimeWaitNotice>,
}

impl RuntimeClaims for FileRuntimeClaims {
    fn claim(&self) -> Result<Arc<dyn RuntimeOwner>> {
        let file = self.open()?;
        FileExt::try_lock_exclusive(&file).context("claim image cache ownership")?;
        Ok(Arc::new(LockedRuntime { file }))
    }

    fn wait(&self) -> Result<Arc<dyn RuntimeOwner>> {
        let file = self.open()?;
        match FileExt::try_lock_exclusive(&file) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                self.notice.waiting();
                FileExt::lock_exclusive(&file).context("wait for runtime ownership")?;
            }
            Err(error) => return Err(error).context("claim runtime ownership"),
        }
        Ok(Arc::new(LockedRuntime { file }))
    }
}

impl FileRuntimeClaims {
    fn open(&self) -> Result<File> {
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(self.control.join("images.lock"))
            .context("open image ownership lock")
    }
}
