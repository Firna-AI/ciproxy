//! The agent's own APFS volume, mounted at `/var/db/ciproxy/area`.
//!
//! The volume has a quota, so jobs and images cannot fill the Mac's disk.
//! Install creates it in the container of the startup disk, or in the
//! container that `CIPROXY_AREA_DISK` names, such as an external SSD.
//! `area.json` records its UUID and the UUID of its container. Install and
//! uninstall act only on that volume. An `/etc/fstab` line mounts it again
//! at boot, and when its disk is connected again.

use std::path::{Path, PathBuf};

use super::area_target::target;
use super::diskutil::{Plist, area_error, diskutil, info_value, new_device, run_checked};
use crate::error::{Error, ErrorResultExt, Result};
pub(crate) use crate::macos::area_access::clear_result;
use crate::macos::area_disk::AreaRecord;
use crate::macos::area_disk::{AreaDisk, Deletion, MISSING_REPAIR, area_mounted, deletion};
use crate::macos::hardware::area_quota;
use crate::macos::paths::{AREA, CONTROL};
use crate::persistence::{atomic_private_write, private_directory};

const NAME: &str = "ciproxy";
const FSTAB: &str = "/etc/fstab";

fn record_path() -> PathBuf {
    Path::new(CONTROL).join("area.json")
}

pub(crate) fn record() -> Result<Option<AreaRecord>> {
    match std::fs::read(record_path()) {
        Ok(bytes) => Ok(Some(
            serde_json::from_slice(&bytes).context("decode the area record")?,
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("read the area record"),
    }
}

/// Create the volume once, or mount the recorded one.
pub(crate) fn prepare(requested_bytes: Option<u64>, disk: &AreaDisk) -> Result<AreaRecord> {
    if let Some(record) = record()? {
        if disk.setting().is_some() && disk.setting() != record.disk {
            tracing::warn!(
                recorded = record.disk.as_deref().unwrap_or("the startup disk"),
                "CIPROXY_AREA_DISK applies to a new area volume only; keep the recorded volume. Uninstall first to move it"
            );
        }
        mount(&record)?;
        return Ok(record);
    }
    let target = target(disk)?;
    if target.external {
        tracing::warn!(
            container = %target.container,
            "the area volume is on an external or removable disk. Keep the disk connected: when it is gone, the Mac takes no jobs and its running jobs stop"
        );
    }
    let Some(quota) = area_quota(target.free, requested_bytes) else {
        return Err(Error::Preparation {
            component: "area volume",
            repair: "Free at least 176 GB on the disk of the area, set CIPROXY_AREA_DISK to another APFS disk, or set CIPROXY_AREA_BYTES to a size that fits, of at least 20 GB. Then rerun install.",
        });
    };
    private_directory(Path::new(AREA))?;
    let output = diskutil(&[
        "apfs",
        "addVolume",
        &target.container,
        "APFS",
        NAME,
        "-quota",
        &format!("{quota}B"),
        "-nomount",
    ])?;
    let device = new_device(&output).ok_or_else(|| area_error("read the new volume"))?;
    let record = AreaRecord {
        volume_uuid: info_value(&device, "VolumeUUID")?,
        quota_bytes: quota,
        container_uuid: target.container_uuid,
        disk: disk.setting(),
        external: target.external,
    };
    atomic_private_write(
        &record_path(),
        &serde_json::to_vec(&record).context("encode the area record")?,
    )?;
    add_fstab_line(&record.volume_uuid)?;
    mount(&record)?;
    // Spotlight does not manage a `nobrowse` volume under /private/var, so
    // `mdutil -i off` fails there with "unknown indexing state". The marker
    // file keeps Spotlight off the volume if it ever looks.
    std::fs::write(Path::new(AREA).join(".metadata_never_index"), b"")
        .context("keep Spotlight off the area volume")?;
    let real = real_area();
    if let Some(error) = run_checked("tmutil", &["addexclusion", "-v", &real.to_string_lossy()]) {
        tracing::warn!(error, "Time Machine can back up the area volume");
    }
    Ok(record)
}

/// Mount the recorded volume when it is not mounted at the area path, and
/// check that it uses file owners.
///
/// A volume that is not there, for example on an unplugged disk, gives the
/// repair to connect it. launchd starts the daemon again every 5 seconds,
/// so the agent goes on when the disk is back. fstab may mount the volume
/// at the same time as this call, so a failed mount counts only when the
/// volume is still not at the area path.
pub(crate) fn mount(record: &AreaRecord) -> Result<()> {
    Plist::info(&record.volume_uuid).map_err(|_| Error::Preparation {
        component: "area volume",
        repair: MISSING_REPAIR,
    })?;
    if !mounted(record) {
        private_directory(Path::new(AREA))?;
        if let Err(error) = diskutil(&["mount", "-mountPoint", AREA, &record.volume_uuid]) {
            tracing::warn!(%error, "the area mount failed; check the mount point again");
        }
        if !mounted(record) {
            return Err(area_error("mount the area volume"));
        }
    }
    ensure_owners(record)
}

/// macOS can ignore file owners on an external volume. Then every user can
/// write the root-only files of the agent. So the agent turns owners on,
/// and refuses a volume where they stay off.
fn ensure_owners(record: &AreaRecord) -> Result<()> {
    let owners =
        |record: &AreaRecord| Plist::info(&record.volume_uuid)?.flag("GlobalPermissionsEnabled");
    match owners(record)? {
        Some(true) => return Ok(()),
        None => {
            tracing::warn!("cannot read whether the area volume uses file owners");
            return Ok(());
        }
        Some(false) => {}
    }
    if let Err(error) = diskutil(&["enableOwnership", &real_area().to_string_lossy()]) {
        tracing::warn!(%error, "cannot turn on file owners on the area volume");
    }
    if owners(record)? == Some(false) {
        return Err(Error::Preparation {
            component: "area volume",
            repair: "Run sudo diskutil enableOwnership /private/var/db/ciproxy/area. If owners stay off, unmount and mount the ciproxy volume. The agent keeps root-only files on its volume, so the volume must use file owners. Then restart the agent.",
        });
    }
    Ok(())
}

/// The volume at the area path is the recorded one.
///
/// `/var` is a link to `/private/var`, and `diskutil` reports the real
/// path. So the agent reads the facts of the volume by its UUID and
/// compares the mount point with the real path of the area. A volume that
/// is already mounted there counts as mounted.
pub(crate) fn mounted(record: &AreaRecord) -> bool {
    info_value(&record.volume_uuid, "MountPoint")
        .is_ok_and(|point| Path::new(&point) == real_area().as_path())
}

/// The area path with `/var` resolved, such as `/private/var/db/ciproxy/area`.
fn real_area() -> PathBuf {
    std::fs::canonicalize(AREA)
        .unwrap_or_else(|_| PathBuf::from(AREA.replacen("/var/", "/private/var/", 1)))
}

/// Delete the recorded volume, its fstab line, and the record. A volume
/// that is not there stays on its disk, with a warning.
pub(crate) fn remove() -> Result<()> {
    let Some(record) = record()? else {
        return Ok(());
    };
    let found = Plist::info(&record.volume_uuid).ok().map(|volume| {
        let name = volume.optional("VolumeName").ok().flatten();
        (name.unwrap_or_default(), container_uuid(&volume))
    });
    let found = found
        .as_ref()
        .map(|(name, container)| (name.as_str(), container.as_deref()));
    match deletion(&record, found, NAME) {
        Deletion::Delete => {
            diskutil(&["apfs", "deleteVolume", &record.volume_uuid])?;
        }
        Deletion::Missing => tracing::warn!(
            volume_uuid = %record.volume_uuid,
            "the ciproxy volume is not connected, so it stays on its disk. Connect the disk and run diskutil apfs deleteVolume with this UUID"
        ),
        Deletion::Foreign => tracing::warn!(
            volume_uuid = %record.volume_uuid,
            "the volume with the recorded UUID has another name or container; keep it"
        ),
        Deletion::Unknown => return Err(area_error("read the container of the area volume")),
    }
    remove_mount_point()?;
    remove_fstab_line(&record.volume_uuid)?;
    clear_result(Path::new(CONTROL))?;
    // The record goes last, so a failed step above repeats on a retry.
    std::fs::remove_file(record_path()).context("remove the area record")
}

/// Remove the mount point. While its disk is unplugged, the folder is a
/// bare folder on the startup disk, and it can hold files that the agent
/// wrote there, so all of it goes. A folder that is still a mount point is
/// only removed when it is empty.
fn remove_mount_point() -> Result<()> {
    let removed = if area_mounted(Path::new(AREA)) {
        std::fs::remove_dir(AREA)
    } else {
        std::fs::remove_dir_all(AREA)
    };
    match removed {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).context("remove the area mount point")
        }
        _ => Ok(()),
    }
}

/// The UUID of the container of a volume, when `diskutil` gives it.
fn container_uuid(volume: &Plist) -> Option<String> {
    let reference = volume.optional("APFSContainerReference").ok().flatten()?;
    Plist::container(&reference)
        .and_then(|list| list.optional("Containers.0.APFSContainerUUID"))
        .ok()
        .flatten()
}

fn fstab_line(uuid: &str) -> String {
    format!("UUID={uuid} {AREA} apfs rw,nobrowse")
}

fn add_fstab_line(uuid: &str) -> Result<()> {
    let text = std::fs::read_to_string(FSTAB).unwrap_or_default();
    if text.lines().any(|line| line.trim() == fstab_line(uuid)) {
        return Ok(());
    }
    let mut next = text;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(&fstab_line(uuid));
    next.push('\n');
    std::fs::write(FSTAB, next).context("add the area to /etc/fstab")
}

fn remove_fstab_line(uuid: &str) -> Result<()> {
    let Ok(text) = std::fs::read_to_string(FSTAB) else {
        return Ok(());
    };
    let kept: String = text
        .lines()
        .filter(|line| line.trim() != fstab_line(uuid))
        .map(|line| format!("{line}\n"))
        .collect();
    if kept != text {
        std::fs::write(FSTAB, kept).context("remove the area from /etc/fstab")?;
    }
    Ok(())
}
