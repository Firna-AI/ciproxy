//! Durable root-only ownership record, sparse file, and mount unit publication.

use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;
use std::sync::Mutex;

use fs2::FileExt;
use uuid::Uuid;

use crate::area::AreaFiles;
use crate::area_types::{AreaRecord, Backing, Inventory, Phase};
use crate::error::{ErrorResultExt, Result};
use crate::host_file_ops::{atomic, directory, metadata, owned_file, ownership, sync};
use crate::host_paths::{AREA, BACKING, CONTROL, MOUNT_PATH, mount_unit};
use crate::host_system::repair;
use crate::state::MachineState;

pub(crate) struct LinuxAreaFiles {
    owner: Mutex<Option<File>>,
}

impl LinuxAreaFiles {
    pub(crate) fn new() -> Self {
        Self {
            owner: Mutex::new(None),
        }
    }
}

const RECORD: &str = "/var/lib/ciproxy/area.json";

impl AreaFiles for LinuxAreaFiles {
    fn claim(&self) -> Result<()> {
        let mut owner = self
            .owner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if owner.is_some() {
            return Ok(());
        }
        directory(Path::new(CONTROL))?;
        let path = Path::new(CONTROL).join("host.lock");
        if let Some(metadata) = metadata(&path)?
            && !owned_file(&metadata)
        {
            return Err(ownership());
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .context("open host preparation lock")?;
        if FileExt::try_lock_exclusive(&file).is_err() {
            return Err(repair(
                "host preparation lock",
                "Wait for the other install or uninstall command. Then retry.",
            ));
        }
        *owner = Some(file);
        Ok(())
    }

    fn inspect(&self) -> Result<Inventory> {
        let control = metadata(Path::new(CONTROL))?.ok_or_else(ownership)?;
        if !control.is_dir()
            || control.uid() != 0
            || control.gid() != 0
            || control.mode() & 0o077 != 0
        {
            return Err(ownership());
        }
        let record = match metadata(Path::new(RECORD))? {
            Some(metadata) if owned_file(&metadata) && metadata.mode() & 0o077 == 0 => Some(
                serde_json::from_slice::<AreaRecord>(&fs::read(RECORD).context("read area owner")?)
                    .context("decode area owner; restore area.json from backup")?,
            ),
            Some(_) => return Err(ownership()),
            None => None,
        };
        let backing = metadata(Path::new(BACKING))?.map(|metadata| Backing {
            device: metadata.dev(),
            inode: metadata.ino(),
            bytes: metadata.len(),
            safe: owned_file(&metadata) && metadata.mode() & 0o077 == 0,
        });
        let mut legacy = false;
        for name in ["images", "jailer"] {
            let path = Path::new(CONTROL).join(name);
            if let Some(metadata) = metadata(&path)? {
                if !metadata.is_dir() {
                    return Err(ownership());
                }
                legacy |= fs::read_dir(path)
                    .context("inspect legacy workload directory")?
                    .next()
                    .is_some();
            }
        }
        let state = Path::new(CONTROL).join("state.json");
        if let Some(metadata) = metadata(&state)? {
            if !owned_file(&metadata) {
                return Err(ownership());
            }
            let state: MachineState =
                serde_json::from_slice(&fs::read(state).context("read machine ownership")?)
                    .context("decode machine ownership")?;
            legacy |= record.is_none() && !state.runners.is_empty();
        }
        let disk = fs2::statvfs(CONTROL).context("measure underlying host filesystem")?;
        let mounts = fs::read_to_string("/proc/self/mountinfo")
            .context("inspect workload free-space mount")?;
        let mounted = mounts
            .lines()
            .any(|line| line.split_whitespace().nth(4) == Some(AREA));
        let area_free = if mounted {
            Some(
                fs2::statvfs(AREA)
                    .context("measure workload free space")?
                    .available_space(),
            )
        } else {
            None
        };
        Ok(Inventory {
            record,
            backing,
            legacy,
            host_bytes: disk.total_space(),
            free_bytes: disk.available_space(),
            area_free,
        })
    }

    fn create(&self, host_bytes: u64, bytes: u64, image_budget: u64) -> Result<AreaRecord> {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .mode(0o600)
            .open(BACKING)
            .context("create new sparse XFS backing file; preserve an existing file")?;
        file.set_len(bytes)
            .context("size new sparse XFS backing file")?;
        file.sync_all().context("sync new sparse backing file")?;
        sync(Path::new(CONTROL))?;
        let metadata = file.metadata().context("identify new backing file")?;
        let record = AreaRecord {
            version: 2,
            uuid: Uuid::new_v4().to_string(),
            source_capacity: host_bytes,
            bytes,
            device: metadata.dev(),
            inode: metadata.ino(),
            phase: Phase::Creating,
            image_budget,
            overhead: 0,
            growth_from: None,
        };
        self.record(&record)?;
        Ok(record)
    }

    fn record(&self, record: &AreaRecord) -> Result<()> {
        atomic(
            Path::new(RECORD),
            &serde_json::to_vec(record).context("encode area owner")?,
            0o600,
        )
    }

    fn grow(&self, record: &AreaRecord, bytes: u64) -> Result<AreaRecord> {
        let inventory = self.inspect()?;
        let file = inventory.backing.ok_or_else(ownership)?;
        let mut recorded = inventory.record.ok_or_else(ownership)?;
        recorded.device = file.device;
        if recorded != *record
            || record.phase != Phase::Creating
            || !file.safe
            || file.device != record.device
            || file.inode != record.inode
            || (file.bytes != record.bytes && record.growth_from != Some(file.bytes))
            || bytes < file.bytes
        {
            return Err(ownership());
        }
        let mut grown = record.clone();
        grown.bytes = bytes;
        grown.growth_from = Some(file.bytes);
        self.record(&grown)?;
        let file = OpenOptions::new()
            .write(true)
            .open(BACKING)
            .context("open unpublished area growth")?;
        file.set_len(bytes)
            .context("grow unpublished XFS backing file")?;
        file.sync_all().context("sync unpublished area growth")?;
        Ok(grown)
    }

    fn unit(&self) -> Result<()> {
        if let Some(metadata) = metadata(Path::new(MOUNT_PATH))? {
            if !owned_file(&metadata)
                || fs::read_to_string(MOUNT_PATH).context("read area unit")? != mount_unit()
            {
                return Err(repair(
                    "mount unit ownership",
                    "Preserve the conflicting unit. Restore ciproxy's mount unit before retrying.",
                ));
            }
            return Ok(());
        }
        atomic(Path::new(MOUNT_PATH), mount_unit().as_bytes(), 0o644)
    }

    fn directories(&self) -> Result<()> {
        directory(Path::new(AREA))?;
        for name in ["images", "jailer", "docker", "cache"] {
            directory(&Path::new(AREA).join(name))?;
        }
        Ok(())
    }

    fn erase(&self, expected: &AreaRecord) -> Result<()> {
        let current = self.inspect()?;
        if current.record.as_ref() != Some(expected) || expected.phase != Phase::Removing {
            return Err(ownership());
        }
        if let Some(file) = current.backing {
            if !file.safe
                || file.device != expected.device
                || file.inode != expected.inode
                || file.bytes != expected.bytes
            {
                return Err(ownership());
            }
            fs::remove_file(BACKING).context("remove owned XFS backing file")?;
        }
        if metadata(Path::new(AREA))?.is_some() {
            fs::remove_dir(AREA)
                .context("remove empty owned mountpoint; preserve hidden or foreign data")?;
        }
        self.unit()?;
        fs::remove_file(MOUNT_PATH).context("remove owned mount unit")?;
        fs::remove_file(RECORD).context("remove area owner record")?;
        sync(Path::new(CONTROL))
    }
}
