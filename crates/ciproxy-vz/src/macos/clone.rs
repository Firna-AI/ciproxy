//! Copy a bundle with an APFS clone.

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use super::configuration::new_machine_identifier;
use crate::bundle::{Bundle, Identity};
use crate::bundle_clone::clone_with;
use crate::error::{Error, Result};
use crate::mac_address::MacAddress;

/// `CLONE_NOFOLLOW` of `<sys/clonefile.h>`: do not follow a symbolic link.
const CLONE_NOFOLLOW: u32 = 0x0001;

impl Identity {
    /// A new random MAC address and a new machine identifier.
    pub fn fresh() -> Result<Self> {
        Ok(Self {
            mac_address: MacAddress::random_local()?,
            machine_identifier: new_machine_identifier(),
        })
    }
}

/// Clone the bundle directory `source` to `target` with `clonefile`, then
/// give the clone `identity`.
///
/// Both paths must be on one APFS volume, and `target` must not exist.
/// The clone takes almost no time and no space at first.
///
/// The source needs the disk, the auxiliary storage, and the hardware
/// model, but no identity files. A released image has none. The clone
/// always gets `identity`, also when the source has an identity.
///
/// Use [`Identity::fresh`] for a VM that needs no saved state. Use the
/// identity of a saved state to restore that state into the clone. Two VMs
/// that run at the same time need two identities.
///
/// The clone copies every file in the directory, also a saved state that
/// the caller put there.
pub fn clone_bundle(source: &Bundle, target: &Path, identity: &Identity) -> Result<Bundle> {
    let from = c_path(source.root())?;
    let to = c_path(target)?;
    clone_with(source, target, identity, || {
        // SAFETY: Both strings are valid and end in a null byte.
        let status = unsafe { libc::clonefile(from.as_ptr(), to.as_ptr(), CLONE_NOFOLLOW) };
        if status == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    })
}

fn c_path(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes()).map_err(|_| Error::InvalidBundle {
        path: path.to_owned(),
        reason: "the path holds a null byte".to_owned(),
    })
}
