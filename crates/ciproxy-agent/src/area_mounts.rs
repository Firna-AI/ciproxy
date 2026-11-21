//! XFS and loop identity checks protect foreign mounts and preserve live VMs.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use crate::area::AreaMounts;
use crate::area_types::{AreaRecord, Filesystem};
use crate::error::{Error, ErrorResultExt, Result};
use crate::host_file_ops::{directory, metadata};
use crate::host_paths::{AREA, BACKING, MOUNT};
use crate::host_system::{HostSystem, repair};
use crate::process::arguments;

pub(crate) struct LinuxAreaMounts {
    pub(crate) system: Arc<dyn HostSystem>,
}

#[async_trait]
impl AreaMounts for LinuxAreaMounts {
    async fn filesystem(&self) -> Result<Option<Filesystem>> {
        let output = self
            .system
            .execute(
                "blkid",
                &arguments(&["-p", "-s", "TYPE", "-s", "UUID", "-o", "export", BACKING]),
            )
            .await?;
        if output.code == 2 {
            return Ok(None);
        }
        if output.code != 0 {
            return Err(mount_repair());
        }
        let text = String::from_utf8(output.stdout).context("read XFS filesystem identity")?;
        let value = |key: &str| {
            text.lines()
                .find_map(|line| line.strip_prefix(key))
                .unwrap_or_default()
                .to_owned()
        };
        Ok(Some(Filesystem {
            kind: value("TYPE="),
            uuid: value("UUID="),
        }))
    }

    async fn format(&self, record: &AreaRecord) -> Result<()> {
        if self.mounted()? {
            return Err(mount_repair());
        }
        self.checked(
            "mkfs.xfs",
            &arguments(&[
                "-q",
                "-b",
                "size=4096",
                "-m",
                &format!("reflink=1,uuid={}", record.uuid),
                "-n",
                "ftype=1",
                "--",
                BACKING,
            ]),
        )
        .await
    }

    fn mounted(&self) -> Result<bool> {
        let bytes = self.system.read("/proc/self/mountinfo")?;
        let text = String::from_utf8(bytes).context("read workload mount table")?;
        for line in text.lines() {
            let Some((left, right)) = line.split_once(" - ") else {
                continue;
            };
            let fields = left.split_whitespace().collect::<Vec<_>>();
            if fields.get(4) != Some(&AREA) {
                continue;
            }
            let mounted = right.split_whitespace().collect::<Vec<_>>();
            let source = mounted.get(1).copied().unwrap_or_default();
            let suffix = source.strip_prefix("/dev/loop").unwrap_or_default();
            let options = fields
                .get(5)
                .copied()
                .unwrap_or_default()
                .split(',')
                .collect::<Vec<_>>();
            if fields.get(3) != Some(&"/")
                || mounted.first() != Some(&"xfs")
                || suffix.is_empty()
                || !suffix.bytes().all(|byte| byte.is_ascii_digit())
                || options
                    .iter()
                    .any(|value| matches!(*value, "ro" | "nodev" | "noexec"))
            {
                return Err(mount_repair());
            }
            let path = format!("/sys/class/block/loop{suffix}/loop/backing_file");
            let backing = self.system.read(&path)?;
            if String::from_utf8_lossy(&backing).trim() != BACKING {
                return Err(mount_repair());
            }
            return Ok(true);
        }
        Ok(false)
    }

    async fn start(&self) -> Result<()> {
        if !self.mounted()? {
            if let Some(metadata) = metadata(Path::new(AREA))?
                && (!metadata.is_dir()
                    || fs::read_dir(AREA)
                        .context("inspect uncovered mountpoint")?
                        .next()
                        .is_some())
            {
                return Err(repair(
                    "uncovered workload mountpoint",
                    "Back up /var/lib/ciproxy/area. Move its stopped data aside before retrying.",
                ));
            }
            directory(Path::new(AREA))?;
        }
        self.reload().await?;
        self.checked("systemctl", &arguments(&["enable", MOUNT]))
            .await?;
        self.checked("systemctl", &arguments(&["start", MOUNT]))
            .await
    }

    async fn reflinks(&self) -> Result<()> {
        let output = self.system.execute("xfs_info", &arguments(&[AREA])).await?;
        if output.code != 0
            || !String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .any(|value| value == "reflink=1")
        {
            return Err(repair(
                "XFS reflinks",
                "Restore the reflink-enabled XFS area from backup. Keep existing job data.",
            ));
        }
        Ok(())
    }

    async fn stop(&self, backing_present: bool) -> Result<()> {
        if self.mounted()? {
            self.checked("systemctl", &arguments(&["stop", MOUNT]))
                .await?;
        }
        if backing_present {
            let remaining = match self
                .system
                .execute("losetup", &arguments(&["--associated", BACKING]))
                .await
            {
                Ok(output) if output.code == 0 => output.stdout,
                _ => {
                    return Err(repair(
                        "loop device inspection",
                        "Restore losetup from util-linux. Then rerun uninstall.",
                    ));
                }
            };
            if !remaining.iter().all(u8::is_ascii_whitespace) {
                return Err(repair(
                    "loop device users",
                    "Stop remaining users of /var/lib/ciproxy/area.xfs. Then rerun uninstall.",
                ));
            }
        }
        self.checked("systemctl", &arguments(&["disable", MOUNT]))
            .await
    }

    async fn reload(&self) -> Result<()> {
        self.checked("systemctl", &arguments(&["daemon-reload"]))
            .await
    }

    fn available(&self) -> Result<u64> {
        Ok(fs2::statvfs(AREA)
            .context("measure mounted XFS overhead")?
            .available_space())
    }

    async fn grow(&self) -> Result<()> {
        let output = self
            .system
            .execute(
                "findmnt",
                &arguments(&["-n", "-o", "SOURCE", "--mountpoint", AREA]),
            )
            .await?;
        let source = String::from_utf8(output.stdout).context("read owned loop source")?;
        if !self.mounted()? || output.code != 0 {
            return Err(mount_repair());
        }
        self.checked("losetup", &arguments(&["--set-capacity", source.trim()]))
            .await?;
        self.checked("xfs_growfs", &arguments(&[AREA])).await
    }
}

impl LinuxAreaMounts {
    async fn checked(&self, program: &str, args: &[String]) -> Result<()> {
        match self.system.execute(program, args).await {
            Ok(output) if output.code == 0 => Ok(()),
            _ => Err(mount_repair()),
        }
    }
}

fn mount_repair() -> Error {
    repair(
        "XFS loop mount",
        "Check the owned mount unit and kernel support. Rerun install. Stop remaining area users before retrying uninstall.",
    )
}

#[cfg(test)]
#[path = "_tests_/area_mounts_tests.rs"]
mod area_mounts_tests;
