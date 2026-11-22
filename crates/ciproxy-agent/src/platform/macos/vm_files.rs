//! The files of a VM helper, an image bundle, and a seat on the area volume.

use std::path::{Path, PathBuf};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_vz::{Bundle, Identity, clone_bundle};

use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::guests::VmFolders;
use crate::macos::image_bundle::folder_name;
use crate::macos::launchd::vm_helper_plist;
use crate::macos::paths::vm_plist;
use crate::macos::vm_spec::VmSpec;
use crate::persistence::{atomic_private_write, private_directory};

/// The file that a saved state keeps inside its bundle, so an APFS clone of
/// the bundle also clones the state for the VM that restores it.
pub(crate) const STATE_FILE: &str = "state.vzvmsave";
/// The VM helper writes this file when a restore fails and it boots cold.
pub(crate) const RESTORE_FAILED: &str = "restore-failed";

/// The checked image bundle of an image. The image store owns this folder.
pub(crate) fn image_bundle(area: &Path, image: &ImageId) -> Bundle {
    Bundle::new(area.join("images").join(folder_name(image)).join("bundle"))
}

/// `seats/N/current` holds the saved state, `seats/N/build` the next one.
pub(crate) fn seat_folder(area: &Path, seat: u16, build: bool) -> PathBuf {
    area.join("seats")
        .join(seat.to_string())
        .join(if build { "build" } else { "current" })
}

/// The fixed identity of a seat, made once and kept with the seat.
pub(crate) fn seat_identity(area: &Path, seat: u16) -> Result<Identity> {
    let bundle = Bundle::new(area.join("seats").join(seat.to_string()).join("identity"));
    if let Ok(identity) = bundle.read_identity() {
        return Ok(identity);
    }
    let identity = Identity::fresh().map_err(vz)?;
    private_directory(bundle.root())?;
    bundle.write_identity(&identity).map_err(vz)?;
    Ok(identity)
}

/// Clone `source` into `target` with `identity`, off the async threads.
pub(crate) async fn clone_with(source: Bundle, target: PathBuf, identity: Identity) -> Result<()> {
    tokio::task::spawn_blocking(move || clone_bundle(&source, &target, &identity))
        .await
        .context("join bundle clone")?
        .map(drop)
        .map_err(vz)
}

/// Write `vm.json` and the helper's launchd file. Both stay root-only.
pub(crate) fn write_helper_files(folders: &VmFolders, local_id: &str, spec: &VmSpec) -> Result<()> {
    atomic_private_write(
        &folders.folder(local_id).join("vm.json"),
        &serde_json::to_vec(spec).context("encode VM file")?,
    )?;
    let plist = vm_plist(local_id);
    if let Some(parent) = plist.parent() {
        private_directory(parent)?;
    }
    atomic_private_write(&plist, vm_helper_plist(local_id).as_bytes())
}

/// Remove the helper's launchd file and VM folder of an unloaded helper.
pub(crate) fn remove_helper_files(folders: &VmFolders, local_id: &str) -> Result<()> {
    remove_path(&vm_plist(local_id))?;
    remove_path(&folders.folder(local_id))
}

pub(crate) fn remove_path(path: &Path) -> Result<()> {
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match result {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).context("remove a VM file")
        }
        _ => Ok(()),
    }
}

pub(crate) fn vz(error: ciproxy_vz::Error) -> Error {
    Error::VmHelper {
        reason: error.to_string(),
    }
}
