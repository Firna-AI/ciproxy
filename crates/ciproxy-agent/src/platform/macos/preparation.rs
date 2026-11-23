//! Host preparation on a Mac: the install check, the control directory,
//! the area volume, and the pf rules. Uninstall removes them again.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use super::area;
use super::host::macos_version;
use super::isolation::Isolation;
use super::launchctl::Launchd;
use super::recovery::remove_orphans;
use crate::boundaries::{Images, Service, Store};
use crate::controller::Agent;
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::area_access::{Access, FULL_DISK_ACCESS_REPAIR, probe, record_result};
use crate::macos::area_disk::AreaDisk;
use crate::macos::guests::VmFolders;
use crate::macos::hardware::install_check;
use crate::macos::paths::{AREA, CONTROL};
use crate::persistence::private_directory;
use crate::preparation::{Preparation, Uninstall, UninstallState, stage_error};
use crate::runtime_ownership::RuntimeOwner;
use crate::terminal_completion::{CompletionAction, UninstallCleanup};

pub(crate) struct MacPreparation {
    pub(crate) isolation: Arc<dyn Isolation>,
    pub(crate) launchd: Arc<dyn Launchd>,
    /// `CIPROXY_AREA_BYTES`, the size of a new area volume.
    pub(crate) area_bytes: Option<u64>,
    /// `CIPROXY_AREA_DISK`, the disk of a new area volume.
    pub(crate) area_disk: AreaDisk,
}

#[async_trait]
impl Preparation for MacPreparation {
    async fn prepare(&self) -> Result<()> {
        install_check(true, true, macos_version()?)?;
        private_directory(Path::new(CONTROL)).map_err(|error| {
            stage_error(
                error,
                "control directory",
                "Restore root ownership and write access to /var/db/ciproxy. Then rerun install.",
            )
        })?;
        private_directory(&Path::new(CONTROL).join("vms"))?;
        area::prepare(self.area_bytes, &self.area_disk).map_err(|error| {
            stage_error(
                error,
                "area volume",
                "Check that the disk of the area is APFS and has free space, and that CIPROXY_AREA_DISK names it. Then rerun install.",
            )
        })?;
        for folder in ["images", "vms", "seats"] {
            private_directory(&Path::new(AREA).join(folder))?;
        }
        self.isolation.ensure().await.map(drop).map_err(|error| {
            stage_error(
                error,
                "pf rules",
                "Check that pfctl works and that no profile blocks pf. Then rerun install.",
            )
        })
    }

    async fn verify(&self) -> Result<()> {
        let record = area::record()?.ok_or(Error::Preparation {
            component: "area volume",
            repair: "Run install again to create the area volume.",
        })?;
        area::mount(&record).map_err(|error| {
            stage_error(
                error,
                "area volume",
                "Mount the ciproxy volume in Disk Utility. Then restart the agent.",
            )
        })?;
        if record.disk.is_some() {
            check_access()?;
        }
        let folders = VmFolders {
            area: PathBuf::from(AREA),
        };
        remove_orphans(self.launchd.as_ref(), &folders).await
    }

    async fn remove(&self) -> Result<()> {
        let folders = VmFolders {
            area: PathBuf::from(AREA),
        };
        remove_orphans(self.launchd.as_ref(), &folders).await?;
        self.isolation.remove().await?;
        area::remove()?;
        match std::fs::remove_dir_all(Path::new(CONTROL).join("vms")) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                Err(error).context("remove VM launchd files")
            }
            _ => Ok(()),
        }
    }
}

/// Check that this process can use an area on another disk, and keep the
/// result for install. See [`crate::macos::area_access`].
fn check_access() -> Result<()> {
    let access = probe(Path::new(AREA))?;
    if let Err(error) = record_result(Path::new(CONTROL), access) {
        tracing::warn!(%error, "install cannot read the area access result");
    }
    match access {
        Access::Open => Ok(()),
        Access::Blocked => Err(Error::Preparation {
            component: "area volume",
            repair: FULL_DISK_ACCESS_REPAIR,
        }),
    }
}

/// The final uninstall: remove at once with no recorded VM, else drain.
pub(crate) struct MacUninstall {
    /// Keep the runtime claim until the files and the identity are gone.
    pub(crate) _runtime: Arc<dyn RuntimeOwner>,
    pub(crate) images: Arc<dyn Images>,
    pub(crate) preparation: Arc<dyn Preparation>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) agent: Arc<dyn Agent>,
}

#[async_trait]
impl Uninstall for MacUninstall {
    async fn start(&self) -> Result<UninstallState> {
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

impl MacUninstall {
    async fn complete(&self) -> Result<()> {
        let cleanup = UninstallCleanup {
            preparation: self.preparation.clone(),
            service: self.service.clone(),
        };
        if self
            .agent
            .complete_removal(CompletionAction::Uninstall(cleanup))
            .await?
        {
            Ok(())
        } else {
            Err(Error::Draining)
        }
    }
}
