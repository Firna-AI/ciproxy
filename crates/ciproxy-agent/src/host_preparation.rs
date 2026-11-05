//! Install preparation and final cleanup have explicit mockable entry points.

use std::sync::Arc;

use async_trait::async_trait;

use crate::area::{AreaFiles, DiskArea};
use crate::boundaries::{Images, Service, Store};
use crate::controller::Agent;
use crate::error::{Error, Result};
use crate::host_packages::Packages;
use crate::host_system::repair;
use crate::host_tools::Tools;

#[cfg_attr(test, unimock::unimock(api = PreparationMock))]
#[async_trait]
pub(crate) trait Preparation: Send + Sync {
    async fn prepare(&self) -> Result<()>;
    async fn verify(&self) -> Result<()>;
    async fn remove(&self) -> Result<()>;
}

pub(crate) struct HostPreparation {
    pub(crate) files: Arc<dyn AreaFiles>,
    pub(crate) packages: Arc<dyn Packages>,
    pub(crate) tools: Arc<dyn Tools>,
    pub(crate) area: Arc<dyn DiskArea>,
}

#[async_trait]
impl Preparation for HostPreparation {
    async fn prepare(&self) -> Result<()> {
        stage(
            self.files.claim(),
            "control directory or lock",
            "Restore root ownership and write access to /var/lib/ciproxy. Then rerun install.",
        )?;
        stage(
            self.packages.prepare().await,
            "host packages",
            "Repair the distribution package manager and kernel support. Then rerun install.",
        )?;
        stage(
            self.tools.prepare().await,
            "Firecracker or jailer",
            "Restore HTTPS access to GitHub and write access to /var/lib/ciproxy/tools. Then rerun install.",
        )?;
        stage(
            self.area.prepare().await,
            "XFS area",
            "Check permissions and free space for /var/lib/ciproxy/area.xfs. Then rerun install.",
        )
    }

    async fn verify(&self) -> Result<()> {
        stage(
            self.area.verify().await,
            "XFS area",
            "Restore the backing file and area.json from backup. Then rerun install.",
        )
    }

    async fn remove(&self) -> Result<()> {
        stage(
            self.area.remove().await,
            "XFS area cleanup",
            "Stop remaining users of /var/lib/ciproxy/area. Then rerun uninstall.",
        )?;
        stage(
            self.tools.remove(),
            "owned Firecracker files",
            "Restore root ownership of /var/lib/ciproxy/tools. Then rerun uninstall.",
        )
    }
}

fn stage(result: Result<()>, component: &'static str, action: &'static str) -> Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(stage_error(error, component, action)),
    }
}

pub(crate) fn stage_error(error: Error, component: &'static str, action: &'static str) -> Error {
    match error {
        error @ Error::Preparation { .. } => error,
        error => {
            tracing::warn!(%error, component, "host preparation needs repair");
            repair(component, action)
        }
    }
}

#[async_trait]
pub(crate) trait Uninstall: Send + Sync {
    async fn finish(&self) -> Result<()>;
}

pub(crate) struct FinalUninstall {
    pub(crate) images: Arc<dyn Images>,
    pub(crate) preparation: Arc<dyn Preparation>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) agent: Arc<dyn Agent>,
}

#[async_trait]
impl Uninstall for FinalUninstall {
    async fn finish(&self) -> Result<()> {
        if !self.agent.finished().await {
            return Err(Error::Draining);
        }
        self.images.purge()?;
        self.preparation.remove().await?;
        self.service.uninstall()?;
        self.store.clear()
    }
}

#[cfg(test)]
#[path = "_tests_/host_preparation_tests.rs"]
mod host_preparation_tests;
