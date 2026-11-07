//! Durable Docker attachment leases and reclaimable repository disk files.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::error::{ErrorResultExt, Result};
use crate::layout::Layout;
use crate::persistence::{atomic_private_write, private_directory, private_write_temporary};
use crate::state::VmRecord;

#[derive(Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
/// Failed writer ownership persists until deletion and the detached phase are durable.
pub(crate) enum AttachmentPhase {
    #[default]
    Preparing,
    Offered,
    Discard,
    Abandon,
    Detached,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Lease {
    pub(crate) repository: String,
    pub(crate) writer: bool,
    pub(crate) bytes: u64,
    #[serde(default)]
    pub(crate) phase: AttachmentPhase,
}

pub(crate) fn load(layout: &Layout, vm: &VmRecord) -> Result<Option<Lease>> {
    let bytes = match fs::read(layout.data.join("docker-leases").join(&vm.local_id)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("read job Docker lease"),
    };
    Ok(Some(
        serde_json::from_slice(&bytes).context("decode job Docker lease")?,
    ))
}

pub(crate) fn leases(layout: &Layout) -> Result<Vec<(String, Lease)>> {
    let directory = layout.data.join("docker-leases");
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).context("read Docker leases"),
    };
    let mut leases = Vec::new();
    for entry in entries {
        let entry = entry.context("read Docker lease identity")?;
        let path = entry.path();
        if path == private_write_temporary(&path) {
            continue;
        }
        let bytes = fs::read(path).context("read Docker lease")?;
        leases.push((
            entry.file_name().to_string_lossy().into_owned(),
            serde_json::from_slice(&bytes).context("decode Docker lease")?,
        ));
    }
    Ok(leases)
}

pub(crate) fn save(layout: &Layout, vm: &VmRecord, lease: &Lease) -> Result<()> {
    let directory = layout.data.join("docker-leases");
    private_directory(&directory)?;
    atomic_private_write(
        &directory.join(&vm.local_id),
        &serde_json::to_vec(lease).context("encode Docker reservation")?,
    )
}

pub(crate) fn held(layout: &Layout) -> Result<u64> {
    Ok(leases(layout)?.iter().map(|(_, lease)| lease.bytes).sum())
}

pub(crate) fn unused(layout: &Layout) -> Result<Vec<(SystemTime, std::path::PathBuf, u64)>> {
    let active = leases(layout)?;
    Ok(bases(layout)?
        .into_iter()
        .filter(|(_, path, _)| {
            !active.iter().any(|(_, lease)| {
                lease.writer
                    && path
                        .file_name()
                        .is_some_and(|name| name == lease.repository.as_str())
            })
        })
        .collect())
}

pub(crate) fn bases(layout: &Layout) -> Result<Vec<(SystemTime, std::path::PathBuf, u64)>> {
    let entries = match fs::read_dir(layout.data.join("docker")) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).context("read repository Docker disks"),
    };
    let mut output = Vec::new();
    for entry in entries {
        let entry = entry.context("read repository disk identity")?;
        let path = entry.path().join("base.ext4");
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error).context("measure unused Docker disk"),
        };
        output.push((
            metadata.modified().context("read Docker last-use time")?,
            entry.path(),
            metadata.blocks() * 512,
        ));
    }
    output.sort_by_key(|(time, _, _)| *time);
    Ok(output)
}
