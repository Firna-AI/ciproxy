//! `import-image` checks its arguments and the copy, then the shared
//! extraction reads the copy as it reads a download.

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::image_extract::extract;
use super::image_import::{copy_checked, parse_image_id, parse_sha256};
use crate::error::Error;

const ROOT: u64 = 3 * 1024 * 1024;

fn hex_digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// A small `macos.tar` in the released layout, with a correct sum file.
fn archive() -> Vec<u8> {
    let mut disk = vec![0_u8; ROOT as usize];
    disk[1024 * 1024 + 7] = 9;
    let compressed = zstd::encode_all(disk.as_slice(), 1).expect("compress");
    let manifest = br#"{"minimum_host_macos":"26.7.2","free_job_bytes":5000000000}"#.to_vec();
    let members: Vec<(&str, Vec<u8>)> = vec![
        ("bundle/disk.img.zst", compressed),
        ("bundle/aux.img", b"aux".to_vec()),
        ("bundle/hardware-model.bin", b"model".to_vec()),
        ("manifest.json", manifest),
        ("measurement.json", b"{}".to_vec()),
        ("upstream-software.json", b"{}".to_vec()),
        ("software-report.json", b"{}".to_vec()),
    ];
    let mut sums: String = members
        .iter()
        .map(|(name, bytes)| format!("{}  {name}\n", hex_digest(bytes)))
        .collect();
    sums.push_str(&format!("{}  bundle/disk.img\n", hex_digest(&disk)));
    let mut builder = tar::Builder::new(Vec::new());
    let sum_member = ("macos-content.sha256", sums.into_bytes());
    for (name, bytes) in members.iter().chain([&sum_member]) {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, name, bytes.as_slice())
            .expect("append");
    }
    builder.into_inner().expect("finish the archive")
}

/// A new folder under the system temporary folder, removed on drop.
struct Folder(PathBuf);

impl Folder {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("ciproxy-import-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).expect("folder");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, bytes: &[u8]) {
    std::fs::File::create(path)
        .and_then(|mut file| file.write_all(bytes))
        .expect("write");
}

#[test]
fn the_sha256_argument_takes_64_hex_digits_in_either_case() {
    let upper = "AB".repeat(32);
    assert_eq!(parse_sha256(&upper), Ok("ab".repeat(32)));
    assert_eq!(parse_sha256(&format!(" {upper}\n")), Ok("ab".repeat(32)));
    for value in ["", "ab", &"a".repeat(63), &"a".repeat(65), &"g".repeat(64)] {
        assert!(parse_sha256(value).is_err(), "{value}");
    }
}

#[test]
fn the_image_id_argument_is_not_empty_and_has_no_control_character() {
    assert_eq!(
        parse_image_id("macos-test-2026-11-22"),
        Ok("macos-test-2026-11-22".to_owned())
    );
    for value in ["", "a\nb", "a\u{0}b"] {
        assert!(parse_image_id(value).is_err(), "{value:?}");
    }
}

#[test]
fn a_checked_copy_replaces_an_old_copy_and_is_private() {
    let folder = Folder::new();
    let (source, target) = (folder.path().join("macos.tar"), folder.path().join("x.tar"));
    write(&source, b"archive bytes");
    write(&target, b"an older partial copy");
    let sha256 = hex_digest(b"archive bytes").to_ascii_uppercase();
    copy_checked(&source, &target, &sha256).expect("copy");
    assert_eq!(std::fs::read(&target).expect("read"), b"archive bytes");
    let mode = std::fs::metadata(&target)
        .expect("mode")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn a_wrong_digest_removes_the_copy_and_keeps_the_source() {
    let folder = Folder::new();
    let (source, target) = (folder.path().join("macos.tar"), folder.path().join("x.tar"));
    write(&source, b"archive bytes");
    let result = copy_checked(&source, &target, &hex_digest(b"other bytes"));
    assert!(matches!(result, Err(Error::ImageVerification)));
    assert!(!target.exists());
    assert_eq!(std::fs::read(&source).expect("read"), b"archive bytes");
}

#[test]
fn a_missing_archive_fails_with_no_copy() {
    let folder = Folder::new();
    let target = folder.path().join("x.tar");
    let result = copy_checked(&folder.path().join("absent.tar"), &target, &"0".repeat(64));
    assert!(result.is_err());
    assert!(!target.exists());
}

#[test]
fn the_shared_extraction_reads_an_imported_copy() {
    let folder = Folder::new();
    let bytes = archive();
    let (source, target) = (folder.path().join("macos.tar"), folder.path().join("x.tar"));
    write(&source, &bytes);
    copy_checked(&source, &target, &hex_digest(&bytes)).expect("copy");
    let pending = folder.path().join("x.pending");
    let manifest = extract(&target, &pending, ROOT).expect("extract");
    assert_eq!(manifest.minimum_host_macos, "26.7.2");
    assert_eq!(manifest.free_job_bytes, 5_000_000_000);
    let disk = std::fs::read(pending.join("bundle/disk.img")).expect("disk");
    assert_eq!(disk.len() as u64, ROOT);
    assert_eq!(disk[1024 * 1024 + 7], 9);
    assert!(!pending.join("bundle/disk.img.zst").exists());
    assert!(matches!(
        extract(&target, &folder.path().join("y.pending"), ROOT + 4096),
        Err(Error::ImageVerification)
    ));
}
