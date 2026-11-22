//! Remove VM helpers and VM files that no saved VM record owns.
//!
//! The agent saves a VM record before it starts the helper. A crash between
//! the two leaves a record with no helper, which the guest bridge ends. A
//! helper with no record comes only from damaged state or a manual start.
//! Startup runs this before the core restores the recorded VMs. Uninstall
//! runs it before it deletes the area, so no build helper still runs.

use std::collections::BTreeSet;
use std::path::Path;

use super::launchctl::Launchd;
use super::vm_files::{remove_path, seat_folder};
use crate::error::{ErrorResultExt, Result};
use crate::macos::guests::VmFolders;
use crate::macos::paths::{CONTROL, valid_local_id, vm_plist};
use crate::macos::seats::SEATS;
use crate::state::MachineState;

/// The local IDs of the saved VM records, or none when the state is absent.
pub(crate) fn recorded_ids() -> Result<Option<BTreeSet<String>>> {
    let bytes = match std::fs::read(Path::new(CONTROL).join("state.json")) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Some(BTreeSet::new()));
        }
        Err(error) => return Err(error).context("read machine state"),
    };
    match serde_json::from_slice::<MachineState>(&bytes) {
        Ok(state) => Ok(Some(
            state
                .runners
                .into_iter()
                .map(|vm| vm.local_id)
                .chain(state.failed_starts)
                .collect(),
        )),
        Err(error) => {
            tracing::warn!(%error, "machine state is unreadable; keep every VM helper");
            Ok(None)
        }
    }
}

/// Stop each helper with no record, then remove its launchd file and folder.
pub(crate) async fn remove_orphans(launchd: &dyn Launchd, folders: &VmFolders) -> Result<()> {
    let Some(recorded) = recorded_ids()? else {
        return Ok(());
    };
    let mut orphans: BTreeSet<String> = launchd
        .loaded_vms()
        .await?
        .into_iter()
        .filter(|id| !recorded.contains(id))
        .collect();
    for folder in [Path::new(CONTROL).join("vms"), folders.area.join("vms")] {
        let entries = match std::fs::read_dir(&folder) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error).context("list VM files"),
        };
        for entry in entries {
            let name = entry.context("read a VM file")?.file_name();
            let name = name.to_string_lossy();
            let id = name.strip_suffix(".plist").unwrap_or(&name);
            if valid_local_id(id) && !recorded.contains(id) {
                orphans.insert(id.to_owned());
            }
        }
    }
    for id in orphans {
        tracing::warn!(local_id = %id, "remove a VM helper with no record");
        launchd.bootout_vm(&id).await?;
        remove_path(&vm_plist(&id))?;
        remove_path(&folders.folder(&id))?;
    }
    // A saved-state build helper has no record, so it ended above. Its
    // unpublished folder goes too.
    for seat in 0..SEATS {
        remove_path(&seat_folder(&folders.area, seat, true))?;
    }
    Ok(())
}
