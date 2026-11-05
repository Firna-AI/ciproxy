//! One owned XFS file. Repairs never resize or format a published data area.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::resources::Resources;
use uuid::Uuid;

use crate::area_types::{AreaRecord, Filesystem, Inventory, Phase};
use crate::error::{Error, Result};
use crate::host_system::repair;

#[cfg_attr(test, unimock::unimock(api = AreaFilesMock))]
pub(crate) trait AreaFiles: Send + Sync {
    fn claim(&self) -> Result<()>;
    fn inspect(&self) -> Result<Inventory>;
    fn create(&self, host_bytes: u64, bytes: u64) -> Result<AreaRecord>;
    fn record(&self, record: &AreaRecord) -> Result<()>;
    fn unit(&self) -> Result<()>;
    fn directories(&self) -> Result<()>;
    fn erase(&self, record: &AreaRecord) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = AreaMountsMock))]
#[async_trait]
pub(crate) trait AreaMounts: Send + Sync {
    async fn filesystem(&self) -> Result<Option<Filesystem>>;
    async fn format(&self, record: &AreaRecord) -> Result<()>;
    fn mounted(&self) -> Result<bool>;
    async fn start(&self) -> Result<()>;
    async fn reflinks(&self) -> Result<()>;
    async fn stop(&self, backing_present: bool) -> Result<()>;
    async fn reload(&self) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = DiskAreaMock))]
#[async_trait]
pub(crate) trait DiskArea: Send + Sync {
    async fn prepare(&self) -> Result<()>;
    async fn verify(&self) -> Result<()>;
    async fn remove(&self) -> Result<()>;
}

pub(crate) struct PersistentArea {
    pub(crate) files: Arc<dyn AreaFiles>,
    pub(crate) mounts: Arc<dyn AreaMounts>,
}

#[async_trait]
impl DiskArea for PersistentArea {
    async fn prepare(&self) -> Result<()> {
        self.files.claim()?;
        let inventory = self.files.inspect()?;
        let existing = validated(&inventory)?;
        let mut record = if let Some(record) = existing {
            if record.phase == Phase::Removing {
                return Err(missing());
            }
            if record.phase == Phase::Creating && self.mounts.filesystem().await?.is_none() {
                self.mounts.format(&record).await?;
            }
            record
        } else {
            let bytes = Resources::for_machine(0, 0, inventory.host_bytes, false).job_disk_bytes;
            let record = self.files.create(inventory.host_bytes, bytes)?;
            self.mounts.format(&record).await?;
            record
        };
        self.check_filesystem(&record).await?;
        record.phase = Phase::Ready;
        self.files.record(&record)?;
        self.files.unit()?;
        self.mounts.start().await?;
        if !self.mounts.mounted()? {
            return Err(missing());
        }
        self.mounts.reflinks().await?;
        self.files.directories()
    }

    async fn verify(&self) -> Result<()> {
        let record = validated(&self.files.inspect()?)?.ok_or_else(missing)?;
        if record.phase != Phase::Ready {
            return Err(missing());
        }
        self.check_filesystem(&record).await?;
        if !self.mounts.mounted()? {
            return Err(missing());
        }
        self.mounts.reflinks().await
    }

    async fn remove(&self) -> Result<()> {
        self.files.claim()?;
        let inventory = self.files.inspect()?;
        let Some(mut record) = validated(&inventory)? else {
            return Ok(());
        };
        if inventory.backing.is_some() {
            let blank_creation =
                record.phase == Phase::Creating && self.mounts.filesystem().await?.is_none();
            if !blank_creation {
                self.check_filesystem(&record).await?;
            }
        }
        self.files.unit()?;
        record.phase = Phase::Removing;
        self.files.record(&record)?;
        self.mounts.stop(inventory.backing.is_some()).await?;
        if self.mounts.mounted()? {
            return Err(missing());
        }
        self.files.erase(&record)?;
        self.mounts.reload().await
    }
}

impl PersistentArea {
    async fn check_filesystem(&self, record: &AreaRecord) -> Result<()> {
        let actual = self.mounts.filesystem().await?;
        if actual
            != Some(Filesystem {
                kind: "xfs".to_owned(),
                uuid: record.uuid.clone(),
            })
        {
            return Err(repair(
                "XFS identity",
                "Restore the backing file and its recorded XFS UUID from backup.",
            ));
        }
        Ok(())
    }
}

fn validated(inventory: &Inventory) -> Result<Option<AreaRecord>> {
    if inventory.legacy {
        return Err(repair(
            "legacy workload placement",
            "Drain the old machine. Wait for its jobs. Follow the legacy migration steps in the host preparation guide.",
        ));
    }
    let Some(record) = &inventory.record else {
        if inventory.backing.is_some() {
            return Err(missing());
        }
        return Ok(None);
    };
    if record.version != 1 || Uuid::parse_str(&record.uuid).is_err() {
        return Err(missing());
    }
    let mut current = record.clone();
    match &inventory.backing {
        Some(file) if file.safe && file.inode == record.inode && file.bytes == record.bytes => {
            current.device = file.device;
        }
        None if record.phase == Phase::Removing => (),
        _ => return Err(missing()),
    }
    Ok(Some(current))
}

fn missing() -> Error {
    repair(
        "owned workload area",
        "Preserve the existing area. Restore damaged ownership data from backup. Rerun install or finish the interrupted uninstall.",
    )
}

#[cfg(test)]
#[path = "_tests_/area_tests.rs"]
mod area_tests;

#[cfg(test)]
#[path = "_tests_/area_repair_tests.rs"]
mod area_repair_tests;

#[cfg(test)]
#[path = "_tests_/area_cleanup_tests.rs"]
mod area_cleanup_tests;
