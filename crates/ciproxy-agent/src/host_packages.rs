//! Distribution package managers supply host tools. Missing KVM stays recoverable.

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;
use crate::host_system::{HostSystem, repair};
use crate::process::arguments;

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
        let release = match self.system.read("/etc/os-release") {
            Ok(release) => release,
            Err(_) => {
                return Err(repair(
                    "distribution metadata",
                    "Restore /etc/os-release for the supported distribution. Then rerun install.",
                ));
            }
        };
        let id = String::from_utf8_lossy(&release)
            .lines()
            .find_map(|line| line.strip_prefix("ID="))
            .unwrap_or_default()
            .trim_matches('"')
            .to_owned();
        let (manager, packages) = match id.as_str() {
            "debian" | "ubuntu" => (
                "apt-get",
                "xfsprogs iproute2 nftables iptables coreutils e2fsprogs util-linux kmod procps ca-certificates",
            ),
            "fedora" | "rhel" | "rocky" | "almalinux" => (
                "dnf",
                "xfsprogs iproute nftables iptables coreutils e2fsprogs util-linux kmod procps-ng ca-certificates",
            ),
            _ => {
                return Err(repair(
                    "distribution",
                    "Use a supported Debian, Ubuntu, Fedora, RHEL, Rocky Linux, or AlmaLinux host with systemd.",
                ));
            }
        };
        if manager == "apt-get" {
            self.checked(manager, &arguments(&["update"]), manager, "Restore apt sources and clear package-manager locks. Run apt-get update. Then rerun install.").await?;
        }
        let mut args = arguments(&["install", "-y"]);
        args.extend(packages.split_whitespace().map(str::to_owned));
        self.checked(manager, &args, manager, "Restore the distribution package sources and clear package-manager locks. Then rerun install.").await?;
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
            self.checked("modprobe", &arguments(&[module]), component, action)
                .await?;
        }
        for module in ["kvm", "vhost_vsock"] {
            match self.system.execute("modprobe", &arguments(&[module])).await {
                Ok(output) if output.code == 0 => (),
                _ => tracing::warn!(
                    module,
                    "An optional VM module is unavailable. Registration can continue."
                ),
            }
        }
        Ok(())
    }
}

impl DistributionPackages {
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
