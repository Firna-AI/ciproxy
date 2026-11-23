//! Missing host tools come from the native package manager. Missing KVM stays recoverable.

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{Result, repair};
use crate::host_requirements::{self, Failure, HOST, Manager};
use crate::host_system::HostSystem;
use crate::process::arguments;

const UNKNOWN: &str = "Install the missing host tools with the distribution package manager: ip, nft, iptables, sysctl, modprobe, mkfs.xfs, xfs_info, xfs_growfs, resize2fs, e2fsck, losetup, blkid, findmnt, and a CA bundle. Then rerun install.";
const STILL_MISSING: &str =
    "Install the host tools that the package manager did not supply. Then rerun install.";
const APT_REFRESH: &str =
    "Restore apt sources and clear package-manager locks. Run apt-get update. Then rerun install.";
const KERNEL_MODULES: &str = "The modules of the running kernel are missing, often after a kernel upgrade. Reboot into the installed kernel. Then rerun install.";

#[cfg_attr(test, unimock::unimock(api = PackagesMock))]
#[async_trait]
pub(crate) trait Packages: Send + Sync {
    async fn prepare(&self) -> Result<()>;
}

pub(crate) struct DistributionPackages {
    pub(crate) system: Arc<dyn HostSystem>,
}

#[async_trait]
impl Packages for DistributionPackages {
    async fn prepare(&self) -> Result<()> {
        self.install_missing().await?;
        self.checked(
            "systemctl",
            &arguments(&["show", "--property=Version", "--value"]),
            "systemd",
            "Boot this host with systemd as PID 1. Then rerun install.",
        )
        .await?;
        if self
            .system
            .read("/sys/fs/cgroup/cgroup.controllers")
            .is_err()
        {
            return Err(repair(
                "cgroup v2",
                "Boot with the unified cgroup v2 hierarchy enabled. Then rerun install.",
            ));
        }
        for (module, component, action) in [
            (
                "loop",
                "loop kernel module",
                "Install the distribution loop kernel module. Run modprobe loop. Then rerun install.",
            ),
            (
                "xfs",
                "XFS kernel module",
                "Install the distribution XFS kernel module. Run modprobe xfs. Then rerun install.",
            ),
        ] {
            if self.modprobe(module).await {
                continue;
            }
            let action = if self.running_kernel_modules_missing() {
                KERNEL_MODULES
            } else {
                action
            };
            return Err(repair(component, action));
        }
        for module in ["kvm", "vhost_vsock"] {
            if !self.modprobe(module).await {
                tracing::warn!(
                    module,
                    "An optional VM module is unavailable. Registration can continue."
                );
            }
        }
        Ok(())
    }
}

impl DistributionPackages {
    /// Install only absent tools, so existing variants such as iptables-legacy stay.
    async fn install_missing(&self) -> Result<()> {
        let missing = host_requirements::missing(self.system.as_ref(), HOST).await?;
        if missing.is_empty() {
            return Ok(());
        }
        let names = host_requirements::names(&missing);
        let manager = self
            .system
            .read("/etc/os-release")
            .ok()
            .and_then(|release| Manager::detect(&release));
        let Some(manager) = manager else {
            tracing::warn!(missing = %names, "No supported package manager can install the missing host tools.");
            return Err(repair("host tools", UNKNOWN));
        };
        tracing::info!(missing = %names, manager = manager.program(), "Installing missing host tools.");
        match manager.install(self.system.as_ref(), &missing).await {
            Ok(()) => (),
            Err(Failure::Refresh) => return Err(repair(manager.program(), APT_REFRESH)),
            Err(Failure::Install) => return Err(repair(manager.program(), manager.repair())),
        }
        let left = host_requirements::missing(self.system.as_ref(), &missing).await?;
        if !left.is_empty() {
            let names = host_requirements::names(&left);
            tracing::warn!(missing = %names, "Package installation did not supply these host tools.");
            return Err(repair("host tools", STILL_MISSING));
        }
        Ok(())
    }

    async fn modprobe(&self, module: &str) -> bool {
        matches!(
            self.system.execute("modprobe", &arguments(&[module])).await,
            Ok(output) if output.code == 0
        )
    }

    /// A kernel upgrade without a reboot can remove the running kernel's modules.
    fn running_kernel_modules_missing(&self) -> bool {
        let Ok(release) = self.system.read("/proc/sys/kernel/osrelease") else {
            return false;
        };
        let release = String::from_utf8_lossy(&release).trim().to_owned();
        !release.is_empty()
            && !release.contains('/')
            && self
                .system
                .read(&format!("/lib/modules/{release}/modules.dep"))
                .is_err()
    }

    async fn checked(
        &self,
        program: &str,
        args: &[String],
        component: &'static str,
        action: &'static str,
    ) -> Result<()> {
        match self.system.execute(program, args).await {
            Ok(output) if output.code == 0 => Ok(()),
            _ => Err(repair(component, action)),
        }
    }
}

#[cfg(test)]
#[path = "_tests_/host_packages_tests.rs"]
mod host_packages_tests;
