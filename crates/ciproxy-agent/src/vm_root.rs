//! A private reflink root grows to its label size without changing the base.

use std::fs::{self, OpenOptions};
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{Error, ErrorResultExt, Result};
use crate::process::{Commands, arguments};

#[cfg_attr(test, unimock::unimock(api = RootFilesMock))]
/// File operations used while preparing a disposable VM root.
pub(crate) trait RootFiles: Send + Sync {
    /// Read the verified base's logical length.
    fn length(&self, path: &Path) -> Result<u64>;
    /// Set the VM copy's logical length without allocating its sparse tail.
    fn extend(&self, path: &Path, bytes: u64) -> Result<()>;
}

/// Filesystem operations on the image and its private copy.
pub(crate) struct LinuxRootFiles;

impl RootFiles for LinuxRootFiles {
    fn length(&self, path: &Path) -> Result<u64> {
        Ok(fs::metadata(path).context("read image disk size")?.len())
    }

    fn extend(&self, path: &Path, bytes: u64) -> Result<()> {
        OpenOptions::new()
            .write(true)
            .open(path)
            .context("open private VM root")?
            .set_len(bytes)
            .context("extend sparse VM root")
    }
}

#[async_trait]
/// Prepare the job root separately from any later sticky Docker attachment.
pub(crate) trait RootDisk: Send + Sync {
    /// Reflink the base and grow its copy to the label's exact logical length.
    ///
    /// Restore the file length after ext4 rounds to whole filesystem blocks.
    async fn prepare(&self, image: &Path, copy: &Path, label_bytes: u64) -> Result<()>;
}

/// Mandatory reflink copying followed by offline ext4 growth on the VM copy.
pub(crate) struct ReflinkRoot {
    /// Reads the base length and extends only the disposable copy.
    pub(crate) files: Arc<dyn RootFiles>,
    /// Runs the host tools supplied by installation.
    pub(crate) commands: Arc<dyn Commands>,
}

#[async_trait]
impl RootDisk for ReflinkRoot {
    async fn prepare(&self, image: &Path, copy: &Path, label_bytes: u64) -> Result<()> {
        let image_bytes = self.files.length(image)?;
        if label_bytes < image_bytes {
            return Err(Error::Capacity);
        }
        self.commands
            .run(
                "cp",
                &arguments(&[
                    "--reflink=always",
                    "--sparse=auto",
                    "--",
                    &image.to_string_lossy(),
                    &copy.to_string_lossy(),
                ]),
                &[],
            )
            .await?;
        if label_bytes > image_bytes {
            self.files.extend(copy, label_bytes)?;
            self.commands
                .run("resize2fs", &arguments(&[&copy.to_string_lossy()]), &[])
                .await?;
            self.files.extend(copy, label_bytes)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/vm_root_tests.rs"]
mod vm_root_tests;

#[cfg(test)]
#[path = "_tests_/vm_root_failure_tests.rs"]
mod vm_root_failure_tests;
