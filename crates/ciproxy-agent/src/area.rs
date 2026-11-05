//! One owned XFS file. Repairs never resize or format a published data area.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::area_types::{AreaRecord, Filesystem, Inventory, Phase};
use crate::error::{Error, Result};
use crate::host_system::repair;
use crate::storage_sizing::{AreaSizing, DEFAULT_IMAGE_BYTES};

#[cfg_attr(test, unimock::unimock(api = AreaFilesMock))]
pub(crate) trait AreaFiles: Send + Sync {
    fn claim(&self) -> Result<()>;
    fn inspect(&self) -> Result<Inventory>;
    fn create(&self, host_bytes: u64, bytes: u64, image_budget: u64) -> Result<AreaRecord>;
    fn grow(&self, record: &AreaRecord, bytes: u64) -> Result<AreaRecord>;
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
    fn available(&self) -> Result<u64>;
    async fn grow(&self) -> Result<()>;
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
    pub(crate) requested_budget: Option<u64>,
}

#[async_trait]
impl DiskArea for PersistentArea {
    async fn prepare(&self) -> Result<()> {
        self.files.claim()?;
        let inventory = self.files.inspect()?;
        let existing = validated(&inventory)?;
        let mut record = if let Some(record) = existing {
            self.check_budget(&record)?;
            if record.phase == Phase::Removing {
                return Err(missing());
            }
            if record.phase == Phase::Creating && self.mounts.filesystem().await?.is_none() {
                self.mounts.format(&record).await?;
            }
            record
        } else {
            let budget = self.requested_budget.unwrap_or(DEFAULT_IMAGE_BYTES);
            let sizing = AreaSizing {
                host: inventory.host_bytes,
                images: budget,
            };
            let record = self
                .files
                .create(inventory.host_bytes, sizing.length(0)?, budget)?;
            self.mounts.format(&record).await?;
            record
        };
        self.check_filesystem(&record).await?;
        if record.growth_from.is_some() {
            record = self.files.grow(&record, record.bytes)?;
        }
        self.files.unit()?;
        self.mounts.start().await?;
        if !self.mounts.mounted()? {
            return Err(missing());
        }
        self.mounts.reflinks().await?;
        if record.growth_from.is_some() {
            self.mounts.grow().await?;
            record.growth_from = None;
            self.files.record(&record)?;
        }
        self.files.directories()?;
        let mut available = self.mounts.available()?;
        if record.phase == Phase::Creating {
            for _ in 0..4 {
                let sizing = AreaSizing {
                    host: record.source_capacity,
                    images: record.image_budget,
                };
                let target = sizing.measured(record.bytes, available)?;
                if target <= record.bytes {
                    record.overhead = record.bytes - sizing.job() - sizing.images;
                    record.phase = Phase::Ready;
                    self.files.record(&record)?;
                    return Ok(());
                }
                record = self.files.grow(&record, target)?;
                self.mounts.grow().await?;
                record.growth_from = None;
                self.files.record(&record)?;
                available = self.mounts.available()?;
            }
            return Err(repair(
                "XFS sizing",
                "Rerun install to finish the owned area growth. Preserve the backing file.",
            ));
        }
        self.files.record(&record)
    }

    async fn verify(&self) -> Result<()> {
        let record = validated(&self.files.inspect()?)?.ok_or_else(missing)?;
        self.check_budget(&record)?;
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
    fn check_budget(&self, record: &AreaRecord) -> Result<()> {
        if record.version != 2 || record.image_budget == 0 {
            return Err(repair(
                "old workload area format",
                "Drain the machine and back up its data. Follow the area replacement steps in the host guide.",
            ));
        }
        if self
            .requested_budget
            .is_some_and(|budget| budget != record.image_budget)
        {
            return Err(repair(
                "image budget changed",
                "Drain the machine and follow the area replacement steps. Keep the recorded budget until replacement.",
            ));
        }
        let expected = AreaSizing {
            host: record.source_capacity,
            images: record.image_budget,
        }
        .length(record.overhead)?;
        if record.phase == Phase::Ready && record.bytes < expected {
            return Err(missing());
        }
        Ok(())
    }

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
    if !matches!(record.version, 1 | 2) || Uuid::parse_str(&record.uuid).is_err() {
        return Err(missing());
    }
    let mut current = record.clone();
    match &inventory.backing {
        Some(file)
            if file.safe
                && file.inode == record.inode
                && (file.bytes == record.bytes
                    || record.phase == Phase::Creating
                        && record.growth_from == Some(file.bytes)) =>
        {
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

#[cfg(test)]
#[path = "_tests_/area_budget_tests.rs"]
mod area_budget_tests;
