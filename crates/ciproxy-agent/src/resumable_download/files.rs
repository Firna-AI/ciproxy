//! The names of a partial download beside its archive, its sidecar, and
//! the recovery of both at start.

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use serde::{Deserialize, Serialize};

use crate::error::{ErrorResultExt, Result};

const PARTIAL: &str = ".partial";
const SIDECAR: &str = ".partial.json";

/// The image that the bytes of a partial file belong to.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct Sidecar {
    image_id: ImageId,
    /// The lower-case hex SHA-256 of the whole archive.
    sha256: String,
    size_bytes: u64,
}

impl Sidecar {
    pub(super) fn of(image: &ImageDownload) -> Self {
        Self {
            image_id: image.image_id.clone(),
            sha256: image.sha256.to_ascii_lowercase(),
            size_bytes: image.size_bytes,
        }
    }
}

/// `NAME.tar.partial` and `NAME.tar.partial.json` beside `NAME.tar`.
pub(super) struct Paths {
    pub(super) partial: PathBuf,
    pub(super) sidecar: PathBuf,
}

impl Paths {
    pub(super) fn of(archive: &Path) -> Self {
        let with = |suffix: &str| {
            let mut path = OsString::from(archive.as_os_str());
            path.push(suffix);
            PathBuf::from(path)
        };
        Self {
            partial: with(PARTIAL),
            sidecar: with(SIDECAR),
        }
    }

    /// The sidecar this partial file had when the last command left it.
    pub(super) fn sidecar(&self) -> Result<Option<Sidecar>> {
        match std::fs::read(&self.sidecar) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes).ok()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).context("read the partial download sidecar"),
        }
    }

    /// A partial file exists, and it is not longer than the image of
    /// `sidecar`.
    pub(super) fn fits(&self, sidecar: &Sidecar) -> Result<bool> {
        match std::fs::metadata(&self.partial) {
            Ok(file) => Ok(file.is_file() && file.len() <= sidecar.size_bytes),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error).context("read the partial download"),
        }
    }

    pub(super) fn remove(&self) -> Result<()> {
        remove_file(&self.partial)?;
        remove_file(&self.sidecar)
    }
}

/// Keep each partial file with a valid sidecar in `folder`, and remove
/// every other partial file and sidecar.
pub(crate) fn recover(folder: &Path) -> Result<()> {
    for archive in partial_archives(folder)? {
        let paths = Paths::of(&archive);
        let kept = match paths.sidecar()? {
            Some(sidecar) => paths.fits(&sidecar)?,
            None => false,
        };
        if !kept {
            paths.remove()?;
        }
    }
    Ok(())
}

/// The `NAME.tar` paths that have a partial file or a sidecar in `folder`.
pub(super) fn partial_archives(folder: &Path) -> Result<Vec<PathBuf>> {
    let mut archives = Vec::new();
    for entry in std::fs::read_dir(folder).context("list images")? {
        let name = entry.context("read an image entry")?.file_name();
        let Some(name) = name.to_str() else { continue };
        let stem = name
            .strip_suffix(SIDECAR)
            .or_else(|| name.strip_suffix(PARTIAL));
        if let Some(stem) = stem.filter(|stem| stem.ends_with(".tar")) {
            archives.push(folder.join(stem));
        }
    }
    Ok(archives)
}

pub(super) fn remove_file(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != ErrorKind::NotFound => {
            Err(error).context("remove a partial download")
        }
        _ => Ok(()),
    }
}
