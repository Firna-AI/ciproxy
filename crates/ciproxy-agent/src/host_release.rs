//! Pinned official release and individually checked executable values.

use std::io::{Cursor, Read};

use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;

use crate::error::{Error, ErrorResultExt, Result};
use crate::host_system::repair;

pub(crate) const VERSION: &str = "1.17.0";
pub(crate) const URL: &str = "https://github.com/firecracker-microvm/firecracker/releases/download/v1.17.0/firecracker-v1.17.0-x86_64.tgz";
pub(crate) const ARCHIVE_SHA256: &str =
    "06094a1108ae9e82aa4c23a775aa92758f53f1175d422270d9d6162cb9ade558";
pub(crate) const FIRECRACKER_SHA256: &str =
    "99ad0f5cd0514a88aad0e9ae8cfdb3cc3b4ab9d190e1194602406c786b5de7a5";
pub(crate) const JAILER_SHA256: &str =
    "65ef226e96f0ceda55ba643f445801ef2cc0ea667ef67cad8ac4f406c9c8434f";
pub(crate) const LIMIT: u64 = 32 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) struct Release {
    pub(crate) archive: &'static str,
    pub(crate) firecracker: &'static str,
    pub(crate) jailer: &'static str,
}

pub(crate) const PINNED: Release = Release {
    archive: ARCHIVE_SHA256,
    firecracker: FIRECRACKER_SHA256,
    jailer: JAILER_SHA256,
};

/// Executables admitted only after archive and individual SHA-256 checks.
pub struct CheckedTools {
    firecracker: Vec<u8>,
    jailer: Vec<u8>,
    notices: Vec<(String, Vec<u8>)>,
}

impl CheckedTools {
    pub(crate) fn from_archive(bytes: &[u8], release: Release) -> Result<Self> {
        verify(bytes, release.archive)?;
        let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));
        let mut firecracker = None;
        let mut jailer = None;
        let mut notices = Vec::new();
        for entry in archive
            .entries()
            .context("read pinned Firecracker archive")?
        {
            let mut entry = entry.context("read pinned release member")?;
            if !entry.header().entry_type().is_file() {
                continue;
            }
            let path = entry.path().context("read pinned release path")?;
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let wanted = matches!(
                name,
                "firecracker-v1.17.0-x86_64"
                    | "jailer-v1.17.0-x86_64"
                    | "LICENSE"
                    | "NOTICE"
                    | "THIRD-PARTY"
            );
            if !wanted {
                continue;
            }
            let name = name.to_owned();
            let mut bytes = Vec::new();
            (&mut entry)
                .take(LIMIT + 1)
                .read_to_end(&mut bytes)
                .context("read pinned executable")?;
            if bytes.len() as u64 > LIMIT {
                return Err(integrity());
            }
            match name.as_str() {
                "firecracker-v1.17.0-x86_64" => {
                    verify(&bytes, release.firecracker)?;
                    firecracker = Some(bytes);
                }
                "jailer-v1.17.0-x86_64" => {
                    verify(&bytes, release.jailer)?;
                    jailer = Some(bytes);
                }
                _ => notices.push((name, bytes)),
            }
        }
        Ok(Self {
            firecracker: firecracker.ok_or_else(integrity)?,
            jailer: jailer.ok_or_else(integrity)?,
            notices,
        })
    }

    pub(crate) fn files(&self) -> Vec<(&str, &[u8], u32)> {
        let mut files = vec![
            ("firecracker", self.firecracker.as_slice(), 0o700),
            ("jailer", self.jailer.as_slice(), 0o700),
        ];
        files.extend(
            self.notices
                .iter()
                .map(|(name, bytes)| (name.as_str(), bytes.as_slice(), 0o600)),
        );
        files
    }
}

pub(crate) fn verify(bytes: &[u8], expected: &str) -> Result<()> {
    if bytes.len() as u64 > LIMIT || hex::encode(Sha256::digest(bytes)) != expected {
        return Err(integrity());
    }
    Ok(())
}

fn integrity() -> Error {
    repair(
        "Firecracker release integrity",
        "Retry the pinned release download from GitHub.",
    )
}
