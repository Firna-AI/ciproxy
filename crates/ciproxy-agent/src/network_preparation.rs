//! Repair only missing network tools before an installed agent uses its network.

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{Result, repair};
use crate::host_packages::Packages;
use crate::host_requirements::{self, Manager, NETWORK};
use crate::host_system::HostSystem;

const REPAIR: &str = "Restore the distribution package sources and the ip, nft, iptables, and sysctl commands. The agent will retry network preparation.";

pub(crate) struct NetworkPackages {
    pub(crate) system: Arc<dyn HostSystem>,
}

#[async_trait]
impl Packages for NetworkPackages {
    async fn prepare(&self) -> Result<()> {
        let system = self.system.as_ref();
        let missing = host_requirements::missing(system, NETWORK).await?;
        if missing.is_empty() {
            return Ok(());
        }
        let release = system.read("/etc/os-release")?;
        let Some(manager) = Manager::detect(&release) else {
            return Err(repair("VM network tools", REPAIR));
        };
        if manager.install(system, &missing).await.is_err()
            || !host_requirements::missing(system, NETWORK)
                .await?
                .is_empty()
        {
            return Err(repair("VM network tools", REPAIR));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/network_preparation_tests.rs"]
mod network_preparation_tests;
