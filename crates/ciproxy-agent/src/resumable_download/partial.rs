//! The open partial file of one image download.

use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

use ciproxy_protocol::images::ImageDownload;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use super::files::{Paths, Sidecar, partial_archives, remove_file};
use crate::error::{Error, ErrorResultExt, Result};
use crate::persistence::atomic_private_write;

const HASH_BUFFER: usize = 1024 * 1024;

/// The open partial file of the running download.
pub(super) struct Partial {
    paths: Paths,
    file: tokio::fs::File,
    digest: Sha256,
    len: u64,
}

impl Partial {
    /// Continue the partial file of `image` when its sidecar matches, and
    /// hash its bytes. Otherwise start an empty one. Remove the partial
    /// files of other images.
    pub(super) async fn open(image: &ImageDownload, archive: &Path) -> Result<Self> {
        let Some(folder) = archive.parent() else {
            return Err(Error::internal_message("the image archive has no folder"));
        };
        for other in partial_archives(folder)? {
            if other != archive {
                Paths::of(&other).remove()?;
            }
        }
        let paths = Paths::of(archive);
        let wanted = Sidecar::of(image);
        if paths.sidecar()?.as_ref() == Some(&wanted) && paths.fits(&wanted)? {
            let partial = paths.partial.clone();
            let (file, digest, len) = tokio::task::spawn_blocking(move || hash(&partial))
                .await
                .context("join the partial download hash")??;
            tracing::info!(
                bytes = len,
                "the image download continues from its partial file"
            );
            let file = tokio::fs::File::from_std(file);
            return Ok(Self {
                paths,
                file,
                digest,
                len,
            });
        }
        paths.remove()?;
        let sidecar = serde_json::to_vec(&wanted).context("encode the partial download sidecar")?;
        atomic_private_write(&paths.sidecar, &sidecar)?;
        let file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&paths.partial)
            .await
            .context("create the partial download")?;
        Ok(Self {
            paths,
            file,
            digest: Sha256::new(),
            len: 0,
        })
    }

    /// The bytes on disk.
    pub(super) fn len(&self) -> u64 {
        self.len
    }

    /// Empty the file for a body that starts at the first byte.
    pub(super) async fn restart(&mut self) -> Result<()> {
        self.file
            .set_len(0)
            .await
            .context("empty the partial download")?;
        self.file
            .rewind()
            .await
            .context("rewind the partial download")?;
        self.digest = Sha256::new();
        self.len = 0;
        Ok(())
    }

    /// Append one chunk. More bytes than `size` delete the partial file.
    pub(super) async fn append(&mut self, chunk: &[u8], size: u64) -> Result<()> {
        let len = self.len.saturating_add(chunk.len() as u64);
        if len > size {
            self.paths.remove()?;
            return Err(Error::ImageVerification);
        }
        self.file
            .write_all(chunk)
            .await
            .context("write image bytes")?;
        self.digest.update(chunk);
        self.len = len;
        Ok(())
    }

    /// Wait until each written byte is in the file, so that the next
    /// command hashes the same bytes.
    pub(super) async fn flush(&mut self) -> Result<()> {
        self.file
            .flush()
            .await
            .context("write the partial download")
    }

    /// Check the byte count and the SHA-256. On a match, the partial file
    /// becomes `archive`. On a mismatch, delete it and its sidecar.
    pub(super) async fn finish(self, image: &ImageDownload, archive: &Path) -> Result<()> {
        let Self {
            paths,
            file,
            digest,
            len,
        } = self;
        file.sync_all().await.context("sync the partial download")?;
        let matches = hex::encode(digest.finalize()).eq_ignore_ascii_case(&image.sha256);
        if len != image.size_bytes || !matches {
            paths.remove()?;
            return Err(Error::ImageVerification);
        }
        std::fs::rename(&paths.partial, archive).context("finish the image download")?;
        remove_file(&paths.sidecar)
    }
}

/// Hash the bytes of a partial file, and leave it open at its end.
fn hash(path: &Path) -> Result<(std::fs::File, Sha256, u64)> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .context("open the partial download")?;
    let (mut digest, mut len) = (Sha256::new(), 0_u64);
    let mut buffer = vec![0_u8; HASH_BUFFER];
    loop {
        let read = file
            .read(&mut buffer)
            .context("read the partial download")?;
        if read == 0 {
            return Ok((file, digest, len));
        }
        digest.update(&buffer[..read]);
        len += read as u64;
    }
}
