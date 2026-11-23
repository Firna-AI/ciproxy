//! The disk of the area volume: the `CIPROXY_AREA_DISK` setting, the checks
//! of the chosen disk, the area record, and the mount check.
//!
//! These parts take facts that `diskutil` gives, so Linux tests run them.

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Where install creates the area volume.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AreaDisk {
    /// The container of the startup disk, `/System/Volumes/Data`.
    Startup,
    /// The container of a mounted APFS volume, such as `/Volumes/NAME`.
    Path(PathBuf),
    /// An APFS container reference, such as `disk5`.
    Container(String),
}

const SETTING_REPAIR: &str = "Set CIPROXY_AREA_DISK to a mounted path such as /Volumes/NAME, or to an APFS container such as disk5. Then rerun install.";

impl AreaDisk {
    /// Read the setting. An absent or empty value keeps the startup disk.
    pub(crate) fn parse(value: Option<&str>) -> Result<Self> {
        let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(Self::Startup);
        };
        if value.starts_with('/') && !value.chars().any(char::is_control) {
            return Ok(Self::Path(PathBuf::from(value)));
        }
        let number = value.strip_prefix("disk").unwrap_or_default();
        if !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()) {
            return Ok(Self::Container(value.to_owned()));
        }
        Err(setting_error())
    }

    /// The setting text that the area record keeps, or none for the startup
    /// disk.
    pub(crate) fn setting(&self) -> Option<String> {
        match self {
            Self::Startup => None,
            Self::Path(path) => Some(path.to_string_lossy().into_owned()),
            Self::Container(container) => Some(container.clone()),
        }
    }
}

pub(crate) fn setting_error() -> Error {
    Error::Preparation {
        component: "area disk setting",
        repair: SETTING_REPAIR,
    }
}

/// The chosen target is not a local disk that `diskutil` knows, for
/// example a network share.
pub(crate) fn not_local() -> Error {
    Error::Preparation {
        component: "area disk",
        repair: "CIPROXY_AREA_DISK names no local APFS container. A network share cannot hold the area. Name the container that diskutil apfs list shows, such as disk5, not its physical disk, or a mounted path. Then rerun install.",
    }
}

pub(crate) fn not_apfs() -> Error {
    Error::Preparation {
        component: "area disk",
        repair: "CIPROXY_AREA_DISK names a volume that is not APFS. Choose an APFS volume, or erase the disk as APFS in Disk Utility. Then rerun install.",
    }
}

/// The `diskutil` facts of a chosen disk. A container target has no volume
/// facts.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DiskFacts {
    /// `FilesystemType` of the chosen volume, such as `apfs`.
    pub(crate) filesystem: Option<String>,
    /// `WritableVolume` of the chosen volume.
    pub(crate) writable_volume: Option<bool>,
    /// `WritableMedia` of the physical store of the container.
    pub(crate) writable_media: bool,
    /// `Internal` of the physical store.
    pub(crate) internal: bool,
    /// `Removable`, `RemovableMedia`, or `Ejectable` of the physical store.
    pub(crate) removable: bool,
    /// `BusProtocol` of the physical store, such as `USB` or `Disk Image`.
    pub(crate) protocol: String,
}

/// Refuse a disk that cannot hold the area. Return true for an external or
/// removable disk, which the caller warns about.
pub(crate) fn check_disk(facts: &DiskFacts) -> Result<bool> {
    if facts
        .filesystem
        .as_deref()
        .is_some_and(|filesystem| !filesystem.eq_ignore_ascii_case("apfs"))
    {
        return Err(not_apfs());
    }
    if facts.writable_volume == Some(false) || !facts.writable_media {
        return Err(Error::Preparation {
            component: "area disk",
            repair: "CIPROXY_AREA_DISK names a read-only disk. Choose a writable APFS disk. Then rerun install.",
        });
    }
    if facts.protocol.eq_ignore_ascii_case("Disk Image") {
        return Err(Error::Preparation {
            component: "area disk",
            repair: "CIPROXY_AREA_DISK names a disk image, which can be on a network share. Choose a physical APFS disk. Then rerun install.",
        });
    }
    Ok(!facts.internal || facts.removable)
}

/// `area.json`: the area volume and where it is.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AreaRecord {
    pub(crate) volume_uuid: String,
    pub(crate) quota_bytes: u64,
    /// The UUID of the APFS container of the volume. A container reference
    /// such as `disk5` can change at each boot, but this UUID does not.
    /// Records from older agents have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) container_uuid: Option<String>,
    /// `CIPROXY_AREA_DISK` at install, or none for the startup disk.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) disk: Option<String>,
    /// The disk was external or removable at install.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) external: bool,
}

/// What uninstall does with the recorded volume.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Deletion {
    /// The volume is there, with the agent's name, in the recorded container.
    Delete,
    /// No volume has the recorded UUID, for example on an unplugged disk.
    Missing,
    /// A volume has the UUID but another name or container. Keep it.
    Foreign,
    /// A volume has the UUID and the name, but `diskutil` gave no container
    /// to compare. Keep the volume and the record, so a retry decides.
    Unknown,
}

/// Decide from the facts of the volume with the recorded UUID: its name and
/// the UUID of its container, when `diskutil` finds it.
pub(crate) fn deletion(
    record: &AreaRecord,
    found: Option<(&str, Option<&str>)>,
    name: &str,
) -> Deletion {
    let Some((found_name, found_container)) = found else {
        return Deletion::Missing;
    };
    if found_name != name {
        return Deletion::Foreign;
    }
    match (&record.container_uuid, found_container) {
        (None, _) => Deletion::Delete,
        (Some(recorded), Some(found)) if recorded.eq_ignore_ascii_case(found) => Deletion::Delete,
        (Some(_), Some(_)) => Deletion::Foreign,
        (Some(_), None) => Deletion::Unknown,
    }
}

/// A volume is mounted at `area` when the folder has another device than
/// its parent. A bare folder, such as the mount point after a disk is
/// unplugged, has the device of its parent.
pub(crate) fn area_mounted(area: &Path) -> bool {
    let Some(parent) = area.parent() else {
        return false;
    };
    match (std::fs::metadata(area), std::fs::metadata(parent)) {
        (Ok(area), Ok(parent)) => area.is_dir() && area.dev() != parent.dev(),
        _ => false,
    }
}

/// The repair when the recorded volume is not there.
pub(crate) const MISSING_REPAIR: &str = "The ciproxy volume is missing. Connect the disk that holds it. The agent tries again every 5 seconds and mounts the volume when the disk is back.";
