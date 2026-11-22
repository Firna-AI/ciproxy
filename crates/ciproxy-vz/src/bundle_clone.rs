//! The steps of a bundle clone that need no macOS.
//!
//! The macOS `clone_bundle` copies the directory with `clonefile(2)`. The
//! checks before the copy and the identity after it live here, so the tests
//! run them on every platform with a plain copy.

use std::path::Path;

use crate::bundle::{Bundle, Identity, io};
use crate::error::Result;

/// Check `source`, copy it to `target` with `copy`, then give the copy
/// `identity`.
///
/// The source needs the disk, the auxiliary storage, and the hardware
/// model. It needs no identity files, so a released image is a valid
/// source. The copy always gets `identity`, also when the source has an
/// identity of its own, so two VMs never share one by accident.
///
/// `copy` must create `target` and fail when it exists. When the identity
/// write fails, the partial copy is removed.
pub(crate) fn clone_with(
    source: &Bundle,
    target: &Path,
    identity: &Identity,
    copy: impl FnOnce() -> std::io::Result<()>,
) -> Result<Bundle> {
    source.check_image_files()?;
    copy().map_err(|error| io("clone the bundle", target, error))?;
    let clone = Bundle::new(target);
    if let Err(error) = clone.write_identity(identity) {
        let _partial_clone_stays = std::fs::remove_dir_all(target);
        return Err(error);
    }
    Ok(clone)
}

#[cfg(test)]
#[path = "_tests_/bundle_clone_tests.rs"]
mod bundle_clone_tests;
