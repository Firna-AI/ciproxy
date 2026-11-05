//! Linux hardware observations. Missing KVM produces zero slots, not rejection.

use std::sync::Arc;

use ciproxy_protocol::reports::{DiskReport, HardwareReport};

use crate::area::AreaFiles;
use crate::boundaries::Host;
use crate::error::{Error, ErrorResultExt, Result};
use crate::host_paths::CONTROL;
use crate::host_system::HostSystem;

pub(crate) struct LinuxHost {
    pub(crate) system: Arc<dyn HostSystem>,
    pub(crate) area: Arc<dyn AreaFiles>,
}

impl Host for LinuxHost {
    fn require_root(&self) -> Result<()> {
        let bytes = self.system.read("/proc/self/status")?;
        let status = String::from_utf8_lossy(&bytes);
        let root = status
            .lines()
            .find(|line| line.starts_with("Uid:"))
            .and_then(|line| line.split_whitespace().nth(2))
            == Some("0");
        if !root || !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            return Err(Error::HostUnsupported);
        }
        Ok(())
    }

    fn hostname(&self) -> Result<String> {
        Ok(
            String::from_utf8(self.system.read("/proc/sys/kernel/hostname")?)
                .context("read host name")?
                .trim()
                .to_owned(),
        )
    }

    fn hardware(&self) -> Result<HardwareReport> {
        let cpuinfo =
            String::from_utf8(self.system.read("/proc/cpuinfo")?).context("read processor")?;
        let meminfo =
            String::from_utf8(self.system.read("/proc/meminfo")?).context("read memory")?;
        let memory_kib = meminfo
            .lines()
            .find(|line| line.starts_with("MemTotal:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("0")
            .parse::<u64>()
            .context("parse total memory")?;
        let processor = cpuinfo
            .lines()
            .find_map(|line| {
                let (key, value) = line.split_once(':')?;
                (key.trim() == "model name").then(|| value.trim().to_owned())
            })
            .unwrap_or_else(|| "Unknown processor".to_owned());
        let threads = cpuinfo
            .lines()
            .filter(|line| line.starts_with("processor\t"))
            .count() as u32;
        let disk = self.area.inspect()?;
        let capacity = disk.record.as_ref().map_or(disk.host_bytes, |record| {
            disk.host_bytes.min(record.source_capacity)
        });
        Ok(HardwareReport {
            processor,
            threads,
            memory_bytes: memory_kib.saturating_mul(1024),
            kvm: self.system.kvm(),
            disks: vec![DiskReport {
                name: CONTROL.to_owned(),
                capacity_bytes: capacity,
                free_bytes: disk
                    .free_bytes
                    .min(disk.area_free.unwrap_or(0))
                    .min(capacity),
            }],
        })
    }
}

#[cfg(test)]
#[path = "_tests_/host_capacity_tests.rs"]
mod host_capacity_tests;
