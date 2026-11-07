//! Host preparation and uninstall routing have explicit mockable entry points.

use std::sync::Arc;

use async_trait::async_trait;

use crate::area::{AreaFiles, DiskArea};
use crate::area_types::Inventory;
use crate::boundaries::{Images, Service, Store};
use crate::controller::Agent;
use crate::error::{Error, Result};
use crate::host_packages::Packages;
use crate::host_system::repair;
use crate::host_tools::Tools;
use crate::terminal_completion::{CompletionAction, UninstallCleanup};

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
    /// Remove an empty machine or prepare its recorded VMs for draining.
    async fn start(&self) -> Result<UninstallState>;
    /// Remove the area and identity only after every recorded VM finishes.
    async fn finish(&self) -> Result<()>;
}

/// Cleanup completes immediately without VMs or waits for their drain loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UninstallState {
    /// Guarded cleanup removed the empty machine.
    Complete,
    /// The caller must drain recorded VMs and then finish cleanup.
    Draining,
}

pub(crate) struct FinalUninstall {
    pub(crate) files: Arc<dyn AreaFiles>,
    pub(crate) images: Arc<dyn Images>,
    pub(crate) preparation: Arc<dyn Preparation>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) agent: Arc<dyn Agent>,
}

#[async_trait]
impl Uninstall for FinalUninstall {
    async fn start(&self) -> Result<UninstallState> {
        uninstall_inventory(self.files.as_ref())?;
        let state = self.store.load()?;
        if state.runners.is_empty() {
            self.agent.remove().await?;
            self.complete().await?;
            return Ok(UninstallState::Complete);
        }
        self.preparation.verify().await?;
        self.images.recover()?;
        self.agent.restore().await?;
        self.agent.remove().await?;
        Ok(UninstallState::Draining)
    }

    async fn finish(&self) -> Result<()> {
        self.complete().await
    }
}

impl FinalUninstall {
    async fn complete(&self) -> Result<()> {
        if self
            .agent
            .complete_removal(CompletionAction::Uninstall(UninstallCleanup {
                preparation: self.preparation.clone(),
                service: self.service.clone(),
            }))
            .await?
        {
            Ok(())
        } else {
            Err(Error::Draining)
        }
    }
}

/// Keep ownership repair instructions consistent before and during uninstall.
pub(crate) fn uninstall_inventory(files: &dyn AreaFiles) -> Result<Inventory> {
    match files.inspect() {
        Ok(inventory) => Ok(inventory),
        Err(error) => Err(stage_error(
            error,
            "area ownership record",
            "Restore area.json and its matching backing file from backup. Then rerun uninstall.",
        )),
    }
}

#[cfg(test)]
#[path = "_tests_/uninstall_fixtures.rs"]
mod uninstall_fixtures;

#[cfg(test)]
#[path = "_tests_/host_preparation_tests.rs"]
mod host_preparation_tests;

#[cfg(test)]
#[path = "_tests_/terminal_uninstall_tests.rs"]
mod terminal_uninstall_tests;
