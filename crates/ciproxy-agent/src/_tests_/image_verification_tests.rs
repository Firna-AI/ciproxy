//! Checked bytes and fixed file members protect the host image boundary.

use std::path::Path;

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::secret::Secret;

use crate::error::Error;
use crate::image_verification::{DownloadDigest, ImageMember};

#[test]
fn only_the_complete_expected_download_passes_checksum_verification() {
    let image = ImageDownload {
        image_id: ImageId("image".to_owned()),
        download_url: Secret::new("https://images.example/image"),
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
        size_bytes: 3,
    };
    for bytes in [b"ab".as_slice(), b"abd", b"abcd"] {
        let mut digest = DownloadDigest::default();
        digest.update(bytes);
        assert!(
            matches!(digest.finish(&image), Err(Error::ImageVerification)),
            "{bytes:?}"
        );
    }
    let mut digest = DownloadDigest::default();
    digest.update(b"a");
    digest.update(b"bc");
    digest.finish(&image).unwrap();
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
