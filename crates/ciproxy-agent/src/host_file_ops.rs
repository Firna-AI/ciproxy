//! Root ownership checks precede mutations. New staging names never follow links.

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::host_system::repair;

pub(crate) fn metadata(path: &Path) -> Result<Option<Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("inspect owned host path"),
    }
}

pub(crate) fn owned_file(metadata: &Metadata) -> bool {
    metadata.is_file()
        && metadata.uid() == 0
        && metadata.gid() == 0
        && metadata.nlink() == 1
        && metadata.mode() & 0o022 == 0
}

pub(crate) fn directory(path: &Path) -> Result<()> {
    match metadata(path)? {
        Some(metadata) if metadata.is_dir() && metadata.uid() == 0 && metadata.gid() == 0 => (),
        Some(_) => return Err(ownership()),
        None => fs::create_dir(path).context("create owned host directory")?,
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .context("protect owned host directory")?;
    Ok(())
}

pub(crate) fn atomic(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    if let Some(metadata) = metadata(path)?
        && !owned_file(&metadata)
    {
        return Err(ownership());
    }
    let temporary = path.with_extension(format!("pending-{}", Uuid::new_v4().simple()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(mode)
        .open(&temporary)
        .context("stage owned host file")?;
    file.write_all(bytes).context("write owned host file")?;
    file.sync_all().context("sync owned host file")?;
    fs::rename(&temporary, path).context("publish owned host file")?;
    sync(path.parent().ok_or_else(ownership)?)
}

pub(crate) fn sync(path: &Path) -> Result<()> {
    File::open(path)
        .context("open owned host directory")?
        .sync_all()
        .context("sync owned host directory")?;
    Ok(())
}

pub(crate) fn ownership() -> Error {
    repair(
        "host path ownership",
        "Restore root ownership in /var/lib/ciproxy. Preserve existing data when replacing symlinks or hard links.",
    )
}
