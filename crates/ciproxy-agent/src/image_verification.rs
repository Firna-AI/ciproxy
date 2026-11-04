//! Streaming checksums and exact, regular-file bundle membership.

use std::path::Path;

use ciproxy_protocol::images::ImageDownload;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

#[derive(Default)]
pub(crate) struct DownloadDigest {
    digest: Sha256,
    bytes: u64,
}

impl DownloadDigest {
    pub(crate) fn update(&mut self, bytes: &[u8]) {
        self.digest.update(bytes);
        self.bytes = self.bytes.saturating_add(bytes.len() as u64);
    }

    pub(crate) fn finish(self, image: &ImageDownload) -> Result<()> {
        if self.bytes != image.size_bytes
            || !hex::encode(self.digest.finalize()).eq_ignore_ascii_case(&image.sha256)
        {
            return Err(Error::ImageVerification);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageMember {
    Kernel,
    Rootfs,
}

impl ImageMember {
    pub(crate) fn parse(path: &Path, regular: bool) -> Result<Self> {
        if !regular {
            return Err(Error::ImageLayout);
        }
        match path.to_str() {
            Some("vmlinux") => Ok(Self::Kernel),
            Some("rootfs.ext4") => Ok(Self::Rootfs),
            _ => Err(Error::ImageLayout),
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Kernel => "vmlinux",
            Self::Rootfs => "rootfs.ext4",
        }
    }
}

#[cfg(test)]
#[path = "_tests_/image_verification_tests.rs"]
mod image_verification_tests;
