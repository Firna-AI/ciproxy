//! Extract a checked `macos.tar` into a pending image folder.
//!
//! The disk is decompressed with each all-zero block left as a hole. Every
//! file is hashed while it is written, so the content check needs no
//! second read of the 200 GB disk.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::hardware::MacosVersion;
use crate::macos::image_bundle::{
    CONTENT_SUMS, DISK, DISK_ARCHIVE, MANIFEST, MEMBERS, Manifest, SparseWriter, parse_sums,
    summed_files,
};
use crate::persistence::private_directory;

/// Extract `archive` into `pending` and return the checked manifest.
pub(crate) fn extract(archive: &Path, pending: &Path, root_disk_bytes: u64) -> Result<Manifest> {
    if pending.exists() {
        std::fs::remove_dir_all(pending).context("remove an old pending image")?;
    }
    private_directory(&pending.join("bundle"))?;
    let mut tar = tar::Archive::new(File::open(archive).context("open the image archive")?);
    let mut seen = BTreeSet::new();
    let mut digests = BTreeMap::new();
    for entry in tar.entries().context("read the image archive")? {
        let mut entry = entry.context("read an image member")?;
        if !entry.header().entry_type().is_file() {
            return Err(Error::ImageLayout);
        }
        let path = entry.path().context("read a member path")?;
        let name = path.to_str().ok_or(Error::ImageLayout)?.to_owned();
        let member = MEMBERS
            .into_iter()
            .find(|member| *member == name)
            .ok_or(Error::ImageLayout)?;
        if !seen.insert(member) {
            return Err(Error::ImageLayout);
        }
        if member == DISK_ARCHIVE {
            let (archive_digest, disk_digest) = unpack_disk(&mut entry, pending, root_disk_bytes)?;
            digests.insert(DISK_ARCHIVE.to_owned(), archive_digest);
            digests.insert(DISK.to_owned(), disk_digest);
        } else {
            let output = File::create(pending.join(member)).context("create an image member")?;
            let mut writer = Hashing::new(output);
            io::copy(&mut entry, &mut writer).context("write an image member")?;
            digests.insert(member.to_owned(), writer.hex());
        }
    }
    if seen.len() != MEMBERS.len() {
        return Err(Error::ImageLayout);
    }
    let sums = std::fs::read_to_string(pending.join(CONTENT_SUMS)).context("read sums")?;
    let sums = parse_sums(&sums).ok_or(Error::ImageVerification)?;
    if summed_files().any(|file| sums.get(file) != digests.get(file)) {
        return Err(Error::ImageVerification);
    }
    let manifest: Manifest = serde_json::from_slice(
        &std::fs::read(pending.join(MANIFEST)).context("read the image manifest")?,
    )
    .map_err(|_| Error::ImageLayout)?;
    MacosVersion::parse(&manifest.minimum_host_macos).ok_or(Error::ImageLayout)?;
    std::fs::remove_file(pending.join(DISK_ARCHIVE)).ok();
    Ok(manifest)
}

/// Decompress the disk. Return the digests of the compressed member and of
/// the disk. The disk must have exactly the released root length.
fn unpack_disk(member: &mut impl Read, pending: &Path, length: u64) -> Result<(String, String)> {
    let mut compressed = Hashing::new(member);
    let mut decoder =
        zstd::stream::read::Decoder::new(&mut compressed).context("open the disk stream")?;
    let disk = File::create(pending.join(DISK)).context("create the disk")?;
    let mut output = Hashing::new(SparseWriter::new(disk));
    io::copy(&mut decoder, &mut output).context("decompress the disk")?;
    drop(decoder);
    io::copy(&mut compressed, &mut io::sink()).context("read the disk member end")?;
    let disk_digest = output.hex();
    let (disk, written) = output.inner.finish().context("finish the disk")?;
    if written != length {
        return Err(Error::ImageVerification);
    }
    disk.set_len(length).context("set the disk length")?;
    disk.sync_all().context("sync the disk")?;
    Ok((compressed.hex(), disk_digest))
}

/// Hash every byte that passes through, in either direction.
struct Hashing<T> {
    inner: T,
    digest: Sha256,
}

impl<T> Hashing<T> {
    fn new(inner: T) -> Self {
        Self {
            inner,
            digest: Sha256::new(),
        }
    }

    fn hex(&self) -> String {
        hex::encode(self.digest.clone().finalize())
    }
}

impl<T: Write> Write for Hashing<T> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(bytes)?;
        self.digest.update(&bytes[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Read> Read for Hashing<T> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(bytes)?;
        self.digest.update(&bytes[..read]);
        Ok(read)
    }
}
