//! Fresh reflink root and empty Docker disks live only inside their own jail.

use std::fs::{self, File};
use std::os::unix::fs::{PermissionsExt, chown};
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{Error, ErrorResultExt, Result};
use crate::layout::{Layout, jail_uid};
use crate::persistence::private_directory;
use crate::process::{Commands, arguments};
use crate::state::VmRecord;
use crate::vm_config::VmConfig;

#[cfg_attr(test, unimock::unimock(api = VmFilesMock))]
#[async_trait]
pub(crate) trait VmFiles: Send + Sync {
    async fn prepare(&self, vm: &VmRecord) -> Result<()>;
    fn remove(&self, vm: &VmRecord) -> Result<()>;
}

pub(crate) struct LinuxVmFiles {
    pub(crate) layout: Layout,
    pub(crate) commands: Arc<dyn Commands>,
}

#[async_trait]
impl VmFiles for LinuxVmFiles {
    async fn prepare(&self, vm: &VmRecord) -> Result<()> {
        let root = self.layout.jail_root(vm);
        private_directory(&root)?;
        let image = self.layout.image(&vm.report.image_id);
        let root_size = fs::metadata(image.join("rootfs.ext4"))
            .context("read image disk size")?
            .len();
        let docker_size = vm
            .report
            .size
            .disk_bytes
            .checked_sub(root_size)
            .ok_or(Error::Capacity)?;
        if docker_size == 0 {
            return Err(Error::Capacity);
        }
        self.commands
            .run(
                "cp",
                &arguments(&[
                    "--reflink=always",
                    "--sparse=always",
                    "--",
                    &image.join("rootfs.ext4").to_string_lossy(),
                    &root.join("rootfs.ext4").to_string_lossy(),
                ]),
                &[],
            )
            .await?;
        fs::copy(image.join("vmlinux"), root.join("vmlinux")).context("copy verified kernel")?;
        let docker = File::create(root.join("docker.ext4")).context("create empty Docker disk")?;
        docker
            .set_len(docker_size)
            .context("size empty Docker disk")?;
        self.commands
            .run(
                "mkfs.ext4",
                &arguments(&[
                    "-q",
                    "-F",
                    "-m",
                    "0",
                    &root.join("docker.ext4").to_string_lossy(),
                ]),
                &[],
            )
            .await?;
        let config =
            serde_json::to_vec(&VmConfig::for_vm(vm)).context("serialize VM configuration")?;
        fs::write(root.join("config.json"), config).context("write VM configuration")?;
        for file in ["vmlinux", "rootfs.ext4", "docker.ext4", "config.json"] {
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
