//! Offline Docker filesystem sizing verifies the promised free bytes.

use std::fs::{self, OpenOptions};
use std::os::unix::fs::chown;

use crate::docker::LinuxDockers;
use crate::docker_files::{self, Lease};
use crate::error::{Error, ErrorResultExt, Result};
use crate::layout::jail_uid;
use crate::process::{arguments, reflink_copy};
use crate::state::VmRecord;

pub(crate) struct Preparation {
    pub(crate) base: std::path::PathBuf,
    pub(crate) path: std::path::PathBuf,
    pub(crate) lease: Lease,
    pub(crate) limit: u64,
    pub(crate) data: u64,
    pub(crate) free_pool: u64,
}

impl LinuxDockers {
    pub(crate) async fn prepare_file(&self, vm: &VmRecord, prepared: Preparation) -> Result<()> {
        let Preparation {
            base,
            path,
            mut lease,
            limit,
            data,
            free_pool,
        } = prepared;
        let mut bytes = lease.bytes;
        let base = base.as_path();
        let path = path.as_path();
        let unused = docker_files::unused(&self.layout)?
            .iter()
            .map(|(_, _, bytes)| *bytes)
            .sum::<u64>();
        self.cache
            .budget(
                free_pool
                    .saturating_sub(docker_files::held(&self.layout)?)
                    .saturating_sub(unused),
            )
            .await?;
        let fresh = !base.exists();
        if base.exists() {
            if lease.writer {
                fs::hard_link(base, path).context("bind Docker base into jail")?;
            } else {
                reflink_copy(self.commands.as_ref(), base, path).await?;
            }
        } else {
            let file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
                .context("create Docker filesystem")?;
            file.set_len(bytes).context("size Docker filesystem")?;
            self.commands
                .run(
                    "mkfs.ext4",
                    &arguments(&["-F", "-m", "0", &path.to_string_lossy()]),
                    &[],
                )
                .await?;
            if lease.writer {
                fs::hard_link(path, base).context("retain repository Docker base")?;
            }
        }
        if !fresh {
            match self
                .commands
                .check_filesystem(&path.to_string_lossy())
                .await
            {
                Ok(()) => {}
                Err(Error::DockerFilesystem) => {
                    self.discard_invalid_source(vm, &lease).await?;
                    return Err(Error::DockerFilesystem);
                }
                Err(error) => return Err(error),
            }
        }
        if fs::metadata(path)
            .context("measure prepared Docker filesystem")?
            .len()
            < bytes
        {
            OpenOptions::new()
                .write(true)
                .open(path)
                .context("open granted Docker disk")?
                .set_len(bytes)
                .context("extend granted Docker disk")?;
            self.commands
                .run("resize2fs", &arguments(&[&path.to_string_lossy()]), &[])
                .await?;
        }
        loop {
            let output = self
                .commands
                .run(
                    "dumpe2fs",
                    &arguments(&["-h", &path.to_string_lossy()]),
                    &[],
                )
                .await?;
            let free = filesystem_free(&output)?;
            let needed = vm
                .report
                .requested_size
                .disk_bytes
                .saturating_add(if fresh { data } else { 0 });
            if free >= needed {
                break;
            }
            let next = bytes
                .saturating_add(needed.saturating_sub(free).saturating_mul(2))
                .div_ceil(4096)
                * 4096;
            if next > limit || next <= bytes {
                return Err(Error::Capacity);
            }
            bytes = next;
            lease.bytes = bytes;
            docker_files::save(&self.layout, vm, &lease)?;
            self.cache
                .budget(
                    free_pool
                        .saturating_sub(docker_files::held(&self.layout)?)
                        .saturating_sub(unused),
                )
                .await?;
            OpenOptions::new()
                .write(true)
                .open(path)
                .context("open granted Docker filesystem")?
                .set_len(bytes)
                .context("grow granted Docker disk")?;
            self.commands
                .run("resize2fs", &arguments(&[&path.to_string_lossy()]), &[])
                .await?;
        }
        chown(path, Some(jail_uid(vm)), Some(jail_uid(vm)))
            .context("assign Docker disk to jail")?;
        Ok(())
    }
}

fn filesystem_free(bytes: &[u8]) -> Result<u64> {
    let text = String::from_utf8_lossy(bytes);
    let number = |name: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(name))
            .and_then(|value| value.trim().parse::<u64>().ok())
    };
    Ok(number("Free blocks:")
        .ok_or(Error::Capacity)?
        .saturating_mul(number("Block size:").ok_or(Error::Capacity)?))
}
