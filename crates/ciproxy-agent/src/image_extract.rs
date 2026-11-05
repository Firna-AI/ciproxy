//! Extraction consumes download proof and produces exact bundle proof.

use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;

use flate2::read::GzDecoder;

use crate::error::{Error, ErrorResultExt, Result};
use crate::image_store::LinuxImageStore;
use crate::image_verification::{BundleMembers, CheckedBundle, CheckedDownload};
use crate::persistence::private_directory;
use crate::sparse_copy::{ImageCopy, SparseCopy};

impl LinuxImageStore {
    pub(crate) fn unpack(checked: CheckedDownload) -> Result<CheckedBundle> {
        let root = checked.pending().directory().join("verified");
        private_directory(&root)?;
        let file =
            File::open(checked.pending().archive()).context("open checked image download")?;
        let mut archive = tar::Archive::new(GzDecoder::new(file));
        let mut members = BundleMembers::default();
        for entry in archive.entries().context("read checked image archive")? {
            let mut entry = entry.context("read image member")?;
            let expected = entry.size();
            let member = members.record(
                &entry.path().context("read member path")?,
                entry.header().entry_type().is_file(),
                expected,
            )?;
            let destination = root.join(member.name());
            let mut file = checked.pending().writer(&destination)?;
            let bytes = match SparseCopy.copy(&mut entry, &mut file) {
                Ok(bytes) => bytes,
                Err(error)
                    if error
                        .get_ref()
                        .and_then(|error| error.downcast_ref::<Error>())
                        .is_some_and(|error| matches!(error, Error::ImageBudget)) =>
                {
                    return Err(Error::ImageBudget);
                }
                Err(error) => return Err(error).context("extract image member"),
            };
            if bytes != expected {
                return Err(Error::ImageLayout);
            }
            file.sync()?;
            fs::set_permissions(&destination, fs::Permissions::from_mode(0o400))
                .context("protect image member")?;
        }
        members.finish(checked)
    }
}
