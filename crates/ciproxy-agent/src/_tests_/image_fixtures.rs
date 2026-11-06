//! Typed fake image inputs use the same verifier and member counter as production.

use std::path::{Path, PathBuf};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::{ImageDownload, ImageReport};
use ciproxy_protocol::secret::Secret;

use crate::error::Result;
use crate::fixtures::metadata;
use crate::image_artifact::PendingImage;
use crate::image_verification::{BundleMembers, CheckedBundle, CheckedDownload, DownloadDigest};

pub(super) fn image() -> ImageDownload {
    ImageDownload {
        image_id: ImageId("new".to_owned()),
        download_url: Secret::new("https://images.example/new"),
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
        size_bytes: 3,
        root_disk_bytes: 20,
    }
}

pub(super) fn pending() -> PendingImage {
    PendingImage::new(PathBuf::from("/fixture/images/pending-new"))
}

pub(super) fn digest(bytes: &[u8]) -> DownloadDigest {
    let mut digest = DownloadDigest::default();
    digest.update(bytes);
    digest
}

pub(super) fn catalog() -> ImageReport {
    ImageReport {
        available: vec![metadata("old", 10)],
        newest: Some(ImageId("old".to_owned())),
    }
}

pub(super) fn bundle(checked: CheckedDownload) -> Result<CheckedBundle> {
    let mut members = BundleMembers::default();
    members.record(Path::new("vmlinux"), true, 8)?;
    members.record(Path::new("rootfs.ext4"), true, 20)?;
    members.finish(checked)
}
