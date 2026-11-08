//! Repair only missing network tools before an installed agent uses its network.

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;
use crate::host_packages::Packages;
use crate::host_system::{HostSystem, repair};
use crate::process::arguments;

const REPAIR: &str = "Restore the distribution package sources and the ip, nft, iptables, and sysctl commands. The agent will retry network preparation.";

pub(crate) struct NetworkPackages {
    pub(crate) system: Arc<dyn HostSystem>,
}

impl NetworkPackages {
    async fn missing(&self) -> Result<Vec<&'static str>> {
        let mut missing = Vec::new();
        for tool in ["ip", "nft", "iptables", "sysctl"] {
            let output = self
                .system
                .execute(
                    "sh",
                    &arguments(&["-c", "command -v \"$1\" >/dev/null", "sh", tool]),
                )
                .await?;
            if output.code != 0 {
                missing.push(tool);
            }
        }
        Ok(missing)
    }

    async fn checked(&self, program: &str, args: &[String]) -> Result<()> {
        match self.system.execute(program, args).await {
            Ok(output) if output.code == 0 => Ok(()),
            _ => Err(repair("VM network tools", REPAIR)),
        }
    }
}

#[async_trait]
impl Packages for NetworkPackages {
    async fn prepare(&self) -> Result<()> {
        let missing = self.missing().await?;
        if missing.is_empty() {
            return Ok(());
        }
        let release = self.system.read("/etc/os-release")?;
        let id = String::from_utf8_lossy(&release)
            .lines()
            .find_map(|line| line.strip_prefix("ID="))
            .unwrap_or_default()
            .trim_matches('"')
            .to_owned();
        let (manager, route, process) = match id.as_str() {
            "debian" | "ubuntu" => ("apt-get", "iproute2", "procps"),
            "fedora" | "rhel" | "rocky" | "almalinux" => ("dnf", "iproute", "procps-ng"),
            _ => return Err(repair("VM network tools", REPAIR)),
        };
        let mut args = arguments(&["install", "-y"]);
        if manager == "apt-get" {
            self.checked(manager, &arguments(&["update"])).await?;
            args.push("--no-upgrade".to_owned());
        }
        args.extend(missing.iter().map(|tool| {
            match *tool {
                "ip" => route,
                "nft" => "nftables",
                "sysctl" => process,
                _ => "iptables",
            }
            .to_owned()
        }));
        self.checked(manager, &args).await?;
        if !self.missing().await?.is_empty() {
            return Err(repair("VM network tools", REPAIR));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/network_preparation_tests.rs"]
mod network_preparation_tests;
