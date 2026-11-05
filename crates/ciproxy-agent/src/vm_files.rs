//! One disposable label-sized root holds job files and Docker data in its jail.

use std::fs;
use std::os::unix::fs::{PermissionsExt, chown};
use std::sync::Arc;

use async_trait::async_trait;

use crate::budget_file::BudgetFile;
use crate::error::{ErrorResultExt, Result};
use crate::image_space::{ImageSpace, VM_SUPPORT_BYTES};
use crate::layout::{Layout, jail_uid};
use crate::persistence::private_directory;
use crate::state::VmRecord;
use crate::vm_config::VmConfig;
use crate::vm_root::RootDisk;

#[cfg_attr(test, unimock::unimock(api = VmFilesMock))]
#[async_trait]
pub(crate) trait VmFiles: Send + Sync {
    async fn prepare(&self, vm: &VmRecord) -> Result<()>;
    fn remove(&self, vm: &VmRecord) -> Result<()>;
}

pub(crate) struct LinuxVmFiles {
    pub(crate) layout: Layout,
    pub(crate) root: Arc<dyn RootDisk>,
    pub(crate) space: Arc<dyn ImageSpace>,
}

#[async_trait]
impl VmFiles for LinuxVmFiles {
    async fn prepare(&self, vm: &VmRecord) -> Result<()> {
        let root = self.layout.jail_root(vm);
        let _support = self.space.claim(VM_SUPPORT_BYTES + 256 * 1024)?;
        private_directory(&root)?;
        let image = self.layout.image(&vm.report.image_id);
        self.root
            .prepare(
                &image.join("rootfs.ext4"),
                &root.join("rootfs.ext4"),
                vm.report.size.disk_bytes,
            )
            .await?;
        let mut kernel = BudgetFile::create(&root.join("vmlinux"), self.space.clone())?;
        kernel.copy(&image.join("vmlinux"))?;
        let config =
            serde_json::to_vec(&VmConfig::for_vm(vm)).context("serialize VM configuration")?;
        let mut file = BudgetFile::create(&root.join("config.json"), self.space.clone())?;
        file.write(&config)?;
        file.sync()?;
        for file in ["vmlinux", "rootfs.ext4", "config.json"] {
            let path = root.join(file);
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .context("protect VM files")?;
            chown(&path, Some(jail_uid(vm)), Some(jail_uid(vm)))
                .context("assign VM files to jail")?;
        }
        Ok(())
    }

    fn remove(&self, vm: &VmRecord) -> Result<()> {
        let root = self.layout.jail_root(vm);
        if let Some(directory) = root.parent() {
            match fs::remove_dir_all(directory) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("discard used VM disks"),
            }
        }
        Ok(())
    }
}
