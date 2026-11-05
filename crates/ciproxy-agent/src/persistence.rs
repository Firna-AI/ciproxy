//! Atomic, root-only state replacement on the machine filesystem.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::boundaries::Store;
use crate::error::{ErrorResultExt, Result};
use crate::state::MachineState;

pub(crate) struct FileStore {
    pub(crate) directory: PathBuf,
}

impl Store for FileStore {
    fn load(&self) -> Result<MachineState> {
        match fs::read(self.directory.join("state.json")) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes).context("read machine state")?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(MachineState::default())
            }
            Err(error) => Err(error).context("read machine state"),
        }
    }

    fn save(&self, state: &MachineState) -> Result<()> {
        private_directory(&self.directory)?;
        atomic_private_write(
            &self.directory.join("state.json"),
            &serde_json::to_vec(state).context("serialize machine state")?,
        )
    }

    fn clear(&self) -> Result<()> {
        match fs::remove_file(self.directory.join("state.json")) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error).context("remove machine state"),
        }
    }
}

pub(crate) fn private_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).context("create private directory")?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .context("protect private directory")?;
    Ok(())
}

pub(crate) fn atomic_private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = path.with_extension("pending");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&temporary)
        .context("create private state")?;
    file.write_all(bytes).context("write private state")?;
    file.sync_all().context("sync private state")?;
    fs::rename(&temporary, path).context("replace private state")?;
    if let Some(parent) = path.parent() {
        File::open(parent)
            .context("open state directory")?
            .sync_all()
            .context("sync state directory")?;
    }
    Ok(())
}
