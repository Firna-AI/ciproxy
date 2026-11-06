//! Checked bytes and fixed file members protect the host image boundary.

use std::path::{Path, PathBuf};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::secret::Secret;

use crate::error::Error;
use crate::image_artifact::PendingImage;
use crate::image_fixtures::{digest, image, pending};
use crate::image_verification::{BundleMembers, DownloadDigest, ImageMember};

#[test]
fn only_the_complete_expected_download_passes_checksum_verification() {
    let image = ImageDownload {
        image_id: ImageId("image".to_owned()),
        download_url: Secret::new("https://images.example/image"),
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
        size_bytes: 3,
        root_disk_bytes: 20,
    };
    for bytes in [b"ab".as_slice(), b"abd", b"abcd"] {
        let mut digest = DownloadDigest::default();
        digest.update(bytes);
        assert!(
            matches!(
                digest.finish(&image, PendingImage::new(PathBuf::from("/fixture/pending"))),
                Err(Error::ImageVerification)
            ),
            "{bytes:?}"
        );
    }
    let mut digest = DownloadDigest::default();
    digest.update(b"a");
    digest.update(b"bc");
    digest
        .finish(&image, PendingImage::new(PathBuf::from("/fixture/pending")))
        .unwrap();
}

#[test]
fn bundle_members_cannot_write_paths_or_follow_links() {
    for (path, regular) in [
        ("../vmlinux", true),
        ("/vmlinux", true),
        ("rootfs.ext4", false),
        ("helper.sh", true),
    ] {
        assert!(
            matches!(
                ImageMember::parse(Path::new(path), regular),
                Err(Error::ImageLayout)
            ),
            "{path}"
        );
    }
    assert_eq!(
        ImageMember::parse(Path::new("vmlinux"), true).unwrap(),
        ImageMember::Kernel
    );
    assert_eq!(
        ImageMember::parse(Path::new("rootfs.ext4"), true).unwrap(),
        ImageMember::Rootfs
    );
}

#[test]
fn bundle_completion_requires_both_non_empty_members_exactly_once() {
    for members in [
        vec![],
        vec![("vmlinux", true, 8)],
        vec![("rootfs.ext4", true, 20)],
        vec![
            ("vmlinux", true, 8),
            ("vmlinux", true, 8),
            ("rootfs.ext4", true, 20),
        ],
        vec![
            ("vmlinux", true, 0),
            ("vmlinux", true, 8),
            ("rootfs.ext4", true, 20),
        ],
        vec![
            ("other", true, 8),
            ("vmlinux", true, 8),
            ("rootfs.ext4", true, 20),
        ],
        vec![
            ("vmlinux", false, 8),
            ("vmlinux", true, 8),
            ("rootfs.ext4", true, 20),
        ],
    ] {
        let mut count = BundleMembers::default();
        for (path, regular, bytes) in members {
            let _ = count.record(Path::new(path), regular, bytes);
        }
        let checked = digest(b"abc").finish(&image(), pending()).unwrap();
        assert!(matches!(count.finish(checked), Err(Error::ImageLayout)));
    }
    let mut count = BundleMembers::default();
    count.record(Path::new("vmlinux"), true, 8).unwrap();
    count.record(Path::new("rootfs.ext4"), true, 20).unwrap();
    let checked = digest(b"abc").finish(&image(), pending()).unwrap();
    assert_eq!(count.finish(checked).unwrap().metadata().image_id.0, "new");
    let mut wrong = image();
    wrong.root_disk_bytes = 21;
    let mut count = BundleMembers::default();
    count.record(Path::new("vmlinux"), true, 8).unwrap();
    count.record(Path::new("rootfs.ext4"), true, 20).unwrap();
    let checked = digest(b"abc").finish(&wrong, pending()).unwrap();
    assert!(matches!(
        count.finish(checked),
        Err(Error::ImageVerification)
    ));
}

#[test]
fn released_bundle_metadata_is_accepted_without_allowing_other_members() {
    let mut members = BundleMembers::default();
    for name in [
        "rootfs.ext4",
        "vmlinux",
        "manifest.json",
        "measurement.json",
        "software-report.json",
        "upstream-software.json",
        "firecracker-content.sha256",
    ] {
        assert_eq!(
            members
                .record(
                    Path::new(name),
                    true,
                    if name == "rootfs.ext4" { 20 } else { 8 }
                )
                .map(|member| member.name())
                .map_err(|_| "invalid layout"),
            Ok(name)
        );
    }
    let checked = digest(b"abc").finish(&image(), pending()).unwrap();
    assert_eq!(
        members.finish(checked).unwrap().metadata().image_id.0,
        "new"
    );
}
