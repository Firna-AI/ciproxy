//! The updater installs only a release asset that has a checksum.

use std::path::Path;
use std::sync::Arc;

use cli_updater::BinaryInstaller;

pub(crate) struct CheckedInstaller {
    pub(crate) installer: Arc<dyn BinaryInstaller + Send + Sync>,
}

impl BinaryInstaller for CheckedInstaller {
    fn install(
        &self,
        bytes: &[u8],
        binary: &str,
        current_exe: &Path,
        checksum: Option<&str>,
        asset_name: &str,
    ) -> cli_updater::Result<()> {
        if checksum.is_none() {
            return Err(cli_updater::Error::ChecksumMismatch {
                asset_name: asset_name.to_owned(),
            });
        }
        self.installer
            .install(bytes, binary, current_exe, checksum, asset_name)
    }
}
