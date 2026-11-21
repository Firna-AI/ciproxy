//! Streaming checksums and exact, regular-file bundle membership.

use std::collections::HashSet;
use std::path::Path;

use ciproxy_protocol::images::{ImageDownload, ImageMetadata};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::image_artifact::PendingImage;

/// Proof that the complete download matched its authenticated metadata.
#[derive(Debug)]
pub struct CheckedDownload {
    pending: PendingImage,
    metadata: ImageMetadata,
}

impl CheckedDownload {
    pub(crate) fn pending(&self) -> &PendingImage {
        &self.pending
    }
    pub(crate) fn metadata(&self) -> &ImageMetadata {
        &self.metadata
    }
}

/// Proof that a verified download contains both required non-empty members.
#[derive(Debug)]
pub struct CheckedBundle {
    checked: CheckedDownload,
}

impl CheckedBundle {
    pub(crate) fn pending(&self) -> &PendingImage {
        self.checked.pending()
    }
    pub(crate) fn metadata(&self) -> &ImageMetadata {
        self.checked.metadata()
    }
}

/// Raw download observations. This value grants no right to extract or publish.
#[derive(Default)]
pub struct DownloadDigest {
    digest: Sha256,
    bytes: u64,
}

impl DownloadDigest {
    pub(crate) fn update(&mut self, bytes: &[u8]) {
        self.digest.update(bytes);
        self.bytes = self.bytes.saturating_add(bytes.len() as u64);
    }

    pub(crate) fn exceeds(&self, bytes: u64) -> bool {
        self.bytes > bytes
    }

    pub(crate) fn finish(
        self,
        image: &ImageDownload,
        pending: PendingImage,
    ) -> Result<CheckedDownload> {
        if self.bytes != image.size_bytes
            || !hex::encode(self.digest.finalize()).eq_ignore_ascii_case(&image.sha256)
        {
            return Err(Error::ImageVerification);
        }
        Ok(CheckedDownload {
            pending,
            metadata: ImageMetadata {
                image_id: image.image_id.clone(),
                root_disk_bytes: image.root_disk_bytes,
            },
        })
    }
}

#[derive(Default)]
pub(crate) struct BundleMembers {
    kernel: bool,
    rootfs: bool,
    rootfs_bytes: u64,
    invalid: bool,
    metadata: HashSet<&'static str>,
}

impl BundleMembers {
    pub(crate) fn record(&mut self, path: &Path, regular: bool, bytes: u64) -> Result<ImageMember> {
        let member = match ImageMember::parse(path, regular) {
            Ok(member) => member,
            Err(error) => {
                self.invalid = true;
                return Err(error);
            }
        };
        let seen = match member {
            ImageMember::Kernel => &mut self.kernel,
            ImageMember::Rootfs => &mut self.rootfs,
            ImageMember::Metadata(name) => {
                if bytes == 0 || !self.metadata.insert(name) {
                    self.invalid = true;
                    return Err(Error::ImageLayout);
                }
                return Ok(member);
            }
        };
        if *seen || bytes == 0 {
            self.invalid = true;
            return Err(Error::ImageLayout);
        }
        *seen = true;
        if member == ImageMember::Rootfs {
            self.rootfs_bytes = bytes;
        }
        Ok(member)
    }

    pub(crate) fn finish(self, checked: CheckedDownload) -> Result<CheckedBundle> {
        if self.invalid || !self.kernel || !self.rootfs {
            return Err(Error::ImageLayout);
        }
        if self.rootfs_bytes != checked.metadata.root_disk_bytes {
            return Err(Error::ImageVerification);
        }
        Ok(CheckedBundle { checked })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageMember {
    Kernel,
    Rootfs,
    Metadata(&'static str),
}

impl ImageMember {
    pub(crate) fn parse(path: &Path, regular: bool) -> Result<Self> {
        if !regular {
            return Err(Error::ImageLayout);
        }
        match path.to_str() {
            Some("vmlinux") => Ok(Self::Kernel),
            Some("rootfs.ext4") => Ok(Self::Rootfs),
            Some("upstream-software.json") => Ok(Self::Metadata("upstream-software.json")),
            Some("software-report.json") => Ok(Self::Metadata("software-report.json")),
            Some("measurement.json") => Ok(Self::Metadata("measurement.json")),
            Some("manifest.json") => Ok(Self::Metadata("manifest.json")),
            Some("firecracker-content.sha256") => Ok(Self::Metadata("firecracker-content.sha256")),
            _ => Err(Error::ImageLayout),
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Kernel => "vmlinux",
            Self::Rootfs => "rootfs.ext4",
            Self::Metadata(name) => name,
        }
    }
}

#[cfg(test)]
#[path = "_tests_/image_verification_tests.rs"]
mod image_verification_tests;
