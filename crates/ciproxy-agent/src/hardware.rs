//! Linux hardware observations. Missing KVM produces zero slots, not rejection.

use std::fs::{self, OpenOptions};
use std::path::PathBuf;

use ciproxy_protocol::reports::{DiskReport, HardwareReport};

use crate::boundaries::Host;
use crate::error::{Error, ErrorResultExt, Result};

pub(crate) struct LinuxHost {
    pub(crate) data_directory: PathBuf,
}

impl Host for LinuxHost {
    fn require_root(&self) -> Result<()> {
        let status = fs::read_to_string("/proc/self/status").context("read process identity")?;
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
        Ok(fs::read_to_string("/proc/sys/kernel/hostname")
            .context("read host name")?
            .trim()
            .to_owned())
    }

    fn hardware(&self) -> Result<HardwareReport> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").context("read processor")?;
        let meminfo = fs::read_to_string("/proc/meminfo").context("read memory")?;
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
        let disk = fs2::statvfs(&self.data_directory).context("read data disk space")?;
        Ok(HardwareReport {
            processor,
            threads,
            memory_bytes: memory_kib.saturating_mul(1024),
            kvm: OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/kvm")
                .is_ok(),
            disks: vec![DiskReport {
                name: self.data_directory.display().to_string(),
                capacity_bytes: disk.total_space(),
                free_bytes: disk.available_space(),
            }],
        })
    }
}
