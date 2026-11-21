//! Root-owned runtime versions survive agent updates and retire only after VM cleanup.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::host_file_ops::{atomic, directory, metadata, owned_file, ownership};
use crate::host_paths::{CONTROL, tools};
use crate::host_release::{CheckedTools, FIRECRACKER_SHA256, JAILER_SHA256, LIMIT, verify};
use crate::host_system::repair;
use crate::host_tools::ToolFiles;

pub(crate) struct LinuxToolFiles;

impl ToolFiles for LinuxToolFiles {
    fn valid(&self) -> Result<bool> {
        for path in [
            Path::new(CONTROL).to_owned(),
            Path::new(CONTROL).join("tools"),
            tools(),
        ] {
            let Some(metadata) = metadata(&path)? else {
                return Ok(false);
            };
            if !metadata.is_dir()
                || metadata.uid() != 0
                || metadata.gid() != 0
                || metadata.mode() & 0o077 != 0
            {
                return Err(ownership());
            }
        }
        if !Self::owned(&tools())? {
            return Ok(false);
        }
        for (name, digest) in [
            ("firecracker", FIRECRACKER_SHA256),
            ("jailer", JAILER_SHA256),
        ] {
            let path = tools().join(name);
            let Some(metadata) = metadata(&path)? else {
                return Ok(false);
            };
            if !owned_file(&metadata) {
                return Err(ownership());
            }
            if metadata.mode() & 0o700 != 0o700 {
                return Ok(false);
            }
            if metadata.len() > LIMIT {
                return Ok(false);
            }
            let bytes = fs::read(&path).context("read installed Firecracker tool")?;
            if verify(&bytes, digest).is_err() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn publish(&self, checked: &CheckedTools) -> Result<()> {
        directory(Path::new(CONTROL))?;
        directory(&Path::new(CONTROL).join("tools"))?;
        directory(&tools())?;
        if !Self::owned(&tools())? {
            if fs::read_dir(tools())
                .context("inspect unowned tool directory")?
                .next()
                .is_some()
            {
                return Err(ownership());
            }
            atomic(&tools().join("owner"), b"ciproxy host tools v1\n", 0o600)?;
        }
        for (name, bytes, mode) in checked.files() {
            atomic(&tools().join(name), bytes, mode)?;
        }
        Ok(())
    }

    fn remove(&self) -> Result<()> {
        let root = Path::new(CONTROL).join("tools");
        let Some(metadata) = metadata(&root)? else {
            return Ok(());
        };
        if !metadata.is_dir()
            || metadata.uid() != 0
            || metadata.gid() != 0
            || metadata.mode() & 0o077 != 0
        {
            return Err(ownership());
        }
        for entry in fs::read_dir(&root).context("list owned runtime versions")? {
            let entry = entry.context("read runtime version")?;
            let path = entry.path();
            let name = entry.file_name();
            let version = name
                .to_str()
                .unwrap_or_default()
                .strip_prefix('v')
                .unwrap_or_default();
            let components = version.split('.').collect::<Vec<_>>();
            let release_name = components.len() == 3
                && components
                    .iter()
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
            let metadata = fs::symlink_metadata(&path).context("inspect runtime version")?;
            if !release_name
                || !metadata.is_dir()
                || metadata.uid() != 0
                || metadata.gid() != 0
                || metadata.mode() & 0o077 != 0
                || !Self::owned(&path)?
            {
                tracing::warn!("Unowned runtime files were preserved.");
                continue;
            }
            Self::remove_version(&path)?;
        }
        if fs::read_dir(&root)
            .context("inspect remaining runtime files")?
            .next()
            .is_none()
        {
            fs::remove_dir(root).context("remove empty runtime directory")?;
        }
        Ok(())
    }
}

impl LinuxToolFiles {
    fn remove_version(directory: &Path) -> Result<()> {
        let entries = fs::read_dir(directory)
            .context("list owned runtime files")?
            .collect::<io::Result<Vec<_>>>()
            .context("read owned runtime files")?;
        for entry in &entries {
            let name = entry.file_name();
            let name = name.to_str().unwrap_or_default();
            let known = matches!(
                name,
                "firecracker" | "jailer" | "LICENSE" | "NOTICE" | "THIRD-PARTY" | "owner"
            );
            let staging = name.split_once(".pending-").is_some_and(|(base, id)| {
                matches!(
                    base,
                    "firecracker" | "jailer" | "LICENSE" | "NOTICE" | "THIRD-PARTY" | "owner"
                ) && Uuid::parse_str(id).is_ok()
            });
            if !known && !staging
                || !owned_file(
                    &fs::symlink_metadata(entry.path()).context("inspect owned runtime file")?,
                )
            {
                return Err(repair(
                    "runtime file ownership",
                    "Back up unrecognized files in /var/lib/ciproxy/tools. Move them aside before retrying uninstall.",
                ));
            }
        }
        for entry in entries {
            fs::remove_file(entry.path()).context("remove owned runtime file")?;
        }
        fs::remove_dir(directory).context("remove owned runtime version")?;
        Ok(())
    }

    fn owned(directory: &Path) -> Result<bool> {
        let path = directory.join("owner");
        let Some(metadata) = metadata(&path)? else {
            return Ok(false);
        };
        if !owned_file(&metadata) {
            return Err(ownership());
        }
        if fs::read(path).context("read host tool owner")? != b"ciproxy host tools v1\n" {
            return Err(ownership());
        }
        Ok(true)
    }
}
