//! Find the APFS container of a new area volume and check its disk.

use super::diskutil::{Plist, info_value};
use crate::error::{ErrorResultExt, Result};
use crate::macos::area_disk::{AreaDisk, DiskFacts, check_disk, not_apfs, not_local};

const STARTUP: &str = "/System/Volumes/Data";

/// The APFS container where install creates the volume.
pub(super) struct Target {
    pub(super) container: String,
    pub(super) free: u64,
    pub(super) container_uuid: Option<String>,
    pub(super) external: bool,
}

pub(super) fn target(disk: &AreaDisk) -> Result<Target> {
    let (container, volume) = match disk {
        AreaDisk::Startup => {
            let container = info_value(STARTUP, "APFSContainerReference")?;
            let free = info_value(STARTUP, "APFSContainerFree")?
                .parse()
                .context("read the container free space")?;
            // No container UUID: the record of the startup disk keeps the
            // older form, so an older agent can still read it.
            return Ok(Target {
                container,
                free,
                container_uuid: None,
                external: false,
            });
        }
        AreaDisk::Path(path) => {
            let volume = Plist::info(&path.to_string_lossy()).map_err(|_| not_local())?;
            let filesystem = volume.optional("FilesystemType")?;
            let writable = volume.flag("WritableVolume")?;
            let container = volume
                .optional("APFSContainerReference")?
                .ok_or_else(not_apfs)?;
            (container, (filesystem, writable))
        }
        AreaDisk::Container(container) => (container.clone(), (None, None)),
    };
    let list = Plist::container(&container).map_err(|_| not_local())?;
    if list.value("Containers.0.ContainerReference")? != container {
        return Err(not_local());
    }
    let store = Plist::info(&list.value("Containers.0.PhysicalStores.0.DeviceIdentifier")?)?;
    let removable = ["Removable", "RemovableMedia", "Ejectable"]
        .into_iter()
        .map(|key| store.flag(key))
        .collect::<Result<Vec<_>>>()?
        .contains(&Some(true));
    let external = check_disk(&DiskFacts {
        filesystem: volume.0,
        writable_volume: volume.1,
        writable_media: store.flag("WritableMedia")?.unwrap_or(true),
        internal: store.flag("Internal")?.unwrap_or(false),
        removable,
        protocol: store.optional("BusProtocol")?.unwrap_or_default(),
    })?;
    Ok(Target {
        free: list.number("Containers.0.CapacityFree")?,
        container_uuid: Some(list.value("Containers.0.APFSContainerUUID")?),
        container,
        external,
    })
}
