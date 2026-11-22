//! The released macOS image: its archive members, checks, and local index.
//!
//! One `macos.tar` holds `bundle/disk.img.zst` (zstd with frame checksums),
//! `bundle/aux.img`, `bundle/hardware-model.bin`, four metadata files, and
//! `macos-content.sha256`. The agent writes the identity files of each VM.

use std::collections::BTreeMap;
use std::io::{self, Seek, SeekFrom, Write};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::{ImageMetadata, ImageReport};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const DISK_ARCHIVE: &str = "bundle/disk.img.zst";
pub(crate) const DISK: &str = "bundle/disk.img";
pub(crate) const MANIFEST: &str = "manifest.json";
pub(crate) const CONTENT_SUMS: &str = "macos-content.sha256";

/// Every member of the archive. The agent refuses other paths, links, and
/// duplicates.
pub(crate) const MEMBERS: [&str; 8] = [
    DISK_ARCHIVE,
    "bundle/aux.img",
    "bundle/hardware-model.bin",
    MANIFEST,
    "measurement.json",
    "upstream-software.json",
    "software-report.json",
    CONTENT_SUMS,
];

/// The files that `macos-content.sha256` must name, with the extracted disk
/// in place of nothing.
pub(crate) fn summed_files() -> impl Iterator<Item = &'static str> {
    MEMBERS
        .into_iter()
        .filter(|member| *member != CONTENT_SUMS)
        .chain([DISK])
}

/// Read `sha256sum` lines: 64 hex digits, two spaces or a space and a star,
/// then the path. Each listed file must appear once.
pub(crate) fn parse_sums(text: &str) -> Option<BTreeMap<String, String>> {
    let mut sums = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let (digest, rest) = line.split_at_checked(64)?;
        let path = rest
            .strip_prefix("  ")
            .or_else(|| rest.strip_prefix(" *"))?;
        if !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        if sums
            .insert(path.to_owned(), digest.to_ascii_lowercase())
            .is_some()
        {
            return None;
        }
    }
    summed_files()
        .all(|file| sums.contains_key(file))
        .then_some(sums)
}

/// The part of `manifest.json` that the agent reads.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct Manifest {
    /// The macOS of the guest. A host must run this release or a newer one.
    pub(crate) minimum_host_macos: String,
}

/// The folder name of an image: the hex SHA-256 of its ID, so an ID can
/// never name a path.
pub(crate) fn folder_name(image: &ImageId) -> String {
    hex::encode(Sha256::digest(image.0.as_bytes()))
}

/// `images/index.json`: the published images and the newest one.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImageIndex {
    pub(crate) images: Vec<IndexedImage>,
    pub(crate) newest: Option<ImageId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexedImage {
    pub(crate) image_id: ImageId,
    pub(crate) root_disk_bytes: u64,
    pub(crate) minimum_host_macos: String,
}

impl ImageIndex {
    pub(crate) fn report(&self) -> ImageReport {
        ImageReport {
            available: self
                .images
                .iter()
                .map(|image| ImageMetadata {
                    image_id: image.image_id.clone(),
                    root_disk_bytes: image.root_disk_bytes,
                })
                .collect(),
            newest: self.newest.clone(),
        }
    }

    pub(crate) fn newest(&self) -> Option<&IndexedImage> {
        let newest = self.newest.as_ref()?;
        self.images.iter().find(|image| image.image_id == *newest)
    }

    /// Add or replace an image and make it the newest.
    pub(crate) fn publish(&mut self, image: IndexedImage) {
        self.images.retain(|known| known.image_id != image.image_id);
        self.newest = Some(image.image_id.clone());
        self.images.push(image);
    }

    /// Remove and return every image that is neither the newest nor held.
    pub(crate) fn reclaim(&mut self, held: &[ImageId]) -> Vec<ImageId> {
        let newest = self.newest.clone();
        let (keep, remove) = self.images.drain(..).partition::<Vec<_>, _>(|image| {
            Some(&image.image_id) == newest.as_ref() || held.contains(&image.image_id)
        });
        self.images = keep;
        remove.into_iter().map(|image| image.image_id).collect()
    }
}

/// An all-zero block becomes a hole. The disk then takes only the space of
/// its written blocks.
pub(crate) const SPARSE_BLOCK: usize = 1024 * 1024;

/// Write a stream, but seek over each all-zero block. The caller sets the
/// file length at the end, so a zero tail stays a hole too.
pub(crate) struct SparseWriter<W: Write + Seek> {
    output: W,
    block: Vec<u8>,
    length: u64,
}

impl<W: Write + Seek> SparseWriter<W> {
    pub(crate) fn new(output: W) -> Self {
        Self {
            output,
            block: Vec::with_capacity(SPARSE_BLOCK),
            length: 0,
        }
    }

    /// Flush the last block and return the output and the stream length.
    pub(crate) fn finish(mut self) -> io::Result<(W, u64)> {
        self.flush_block()?;
        Ok((self.output, self.length))
    }

    fn flush_block(&mut self) -> io::Result<()> {
        if self.block.iter().all(|byte| *byte == 0) {
            self.output.seek(SeekFrom::Current(
                i64::try_from(self.block.len()).unwrap_or(0),
            ))?;
        } else {
            self.output.write_all(&self.block)?;
        }
        self.length += self.block.len() as u64;
        self.block.clear();
        Ok(())
    }
}

impl<W: Write + Seek> Write for SparseWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let room = SPARSE_BLOCK - self.block.len();
        let taken = bytes.len().min(room);
        self.block.extend_from_slice(&bytes[..taken]);
        if self.block.len() == SPARSE_BLOCK {
            self.flush_block()?;
        }
        Ok(taken)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}
