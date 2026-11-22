//! `import-image`: put a local `macos.tar` where a download would go.
//!
//! This is a test and operator path for a Mac with no image host. The copy
//! is an APFS clone when the archive is on the area volume. The checks run
//! on the copy, so a later change of the source cannot reach extraction.

use std::fs::File;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use ciproxy_protocol::identifiers::ImageId;
use sha2::{Digest, Sha256};

use crate::error::{Error, ErrorResultExt, Result};

/// One local image to import, from the command line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImageImport {
    /// The `macos.tar` file.
    pub(crate) archive: PathBuf,
    pub(crate) image_id: ImageId,
    /// The lower-case hex SHA-256 of the archive.
    pub(crate) sha256: String,
    /// The released root disk length. The extracted disk must match it.
    pub(crate) root_disk_bytes: u64,
}

/// Accept 64 hex digits, in either case, and give them in lower case.
pub(crate) fn parse_sha256(value: &str) -> std::result::Result<String, String> {
    let value = value.trim();
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value.to_ascii_lowercase())
    } else {
        Err("give the SHA-256 as 64 hex digits".to_owned())
    }
}

/// Accept an image ID that is not empty and has no control character.
pub(crate) fn parse_image_id(value: &str) -> std::result::Result<String, String> {
    if value.is_empty() || value.chars().any(char::is_control) {
        Err("give an image ID that is not empty and has no control character".to_owned())
    } else {
        Ok(value.to_owned())
    }
}

/// Copy `source` to `target` with mode `0600`, then check the size and the
/// SHA-256 of the copy. On a failed check, remove the copy.
///
/// On macOS, `std::fs::copy` makes an APFS clone when both paths are on
/// the same volume, and a full copy when they are not.
pub(crate) fn copy_checked(source: &Path, target: &Path, sha256: &str) -> Result<()> {
    let expected = std::fs::metadata(source)
        .context("read the image archive")?
        .len();
    remove_file(target)?;
    let result = copy_and_check(source, target, expected, sha256);
    if result.is_err() {
        remove_file(target)?;
    }
    result
}

fn copy_and_check(source: &Path, target: &Path, expected: u64, sha256: &str) -> Result<()> {
    let copied = std::fs::copy(source, target).context("copy the image archive")?;
    std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o600))
        .context("restrict the image archive")?;
    let (bytes, digest) = digest(target)?;
    if copied != expected || bytes != expected || !digest.eq_ignore_ascii_case(sha256) {
        return Err(Error::ImageVerification);
    }
    Ok(())
}

/// The byte count and the hex SHA-256 of a file, read in 1 MiB blocks.
fn digest(path: &Path) -> Result<(u64, String)> {
    let mut file = File::open(path).context("open the image archive copy")?;
    let mut buffer = vec![0_u8; 1024 * 1024];
    let (mut hasher, mut bytes) = (Sha256::new(), 0_u64);
    loop {
        let read = file.read(&mut buffer).context("read the image archive")?;
        if read == 0 {
            return Ok((bytes, hex::encode(hasher.finalize())));
        }
        hasher.update(&buffer[..read]);
        bytes += read as u64;
    }
}

fn remove_file(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).context("remove an image archive copy")
        }
        _ => Ok(()),
    }
}
