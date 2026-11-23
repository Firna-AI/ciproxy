//! The clone steps, with a plain copy in place of `clonefile`.

use std::path::Path;

use super::clone_with;
use crate::bundle::{Bundle, Identity};
use crate::error::Error;
use crate::mac_address::MacAddress;

/// A released image: the disk, the auxiliary storage, and the hardware
/// model, with no identity files.
fn image(root: &Path) -> Bundle {
    let image = Bundle::new(root);
    image.create_directory().unwrap();
    image.create_disk(1024 * 1024).unwrap();
    std::fs::write(image.auxiliary_storage(), b"aux").unwrap();
    image.write_hardware_model(b"model").unwrap();
    image
}

/// Copy each file of `from` into a new directory `to`. Like `clonefile`, it
/// fails when `to` exists.
fn copy_directory(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        std::fs::copy(entry.path(), to.join(entry.file_name()))?;
    }
    Ok(())
}

fn seat_identity() -> Identity {
    Identity {
        mac_address: MacAddress::new([0xa2, 0, 0, 0, 0, 1]),
        machine_identifier: b"seat".to_vec(),
    }
}

#[test]
fn an_image_without_identity_files_clones_into_a_complete_bundle() {
    let directory = tempfile::tempdir().unwrap();
    let image = image(&directory.path().join("image"));
    let target = directory.path().join("vm");
    let identity = seat_identity();

    let clone = clone_with(&image, &target, &identity, || {
        copy_directory(image.root(), &target)
    })
    .unwrap();

    clone.check_files().unwrap();
    assert_eq!(clone.root(), target);
    assert_eq!(clone.read_identity().unwrap(), identity);
    assert_eq!(clone.read_hardware_model().unwrap(), b"model");
    assert_eq!(std::fs::read(clone.auxiliary_storage()).unwrap(), b"aux");
    assert_eq!(clone.disk().metadata().unwrap().len(), 1024 * 1024);
    assert!(!image.machine_identifier().exists());
    assert!(!image.mac_address().exists());
}

#[test]
fn a_clone_never_keeps_the_identity_of_its_source() {
    let directory = tempfile::tempdir().unwrap();
    let source = image(&directory.path().join("other-vm"));
    let other = Identity {
        mac_address: MacAddress::new([0xa2, 9, 9, 9, 9, 9]),
        machine_identifier: b"other".to_vec(),
    };
    source.write_identity(&other).unwrap();
    let target = directory.path().join("vm");
    let identity = seat_identity();

    let clone = clone_with(&source, &target, &identity, || {
        copy_directory(source.root(), &target)
    })
    .unwrap();

    assert_eq!(clone.read_identity().unwrap(), identity);
    assert_eq!(source.read_identity().unwrap(), other);
}

#[test]
fn a_source_without_a_hardware_model_fails_before_the_copy() {
    let directory = tempfile::tempdir().unwrap();
    let source = image(&directory.path().join("image"));
    std::fs::remove_file(source.hardware_model()).unwrap();
    let target = directory.path().join("vm");

    let result = clone_with(&source, &target, &seat_identity(), || {
        panic!("the copy must not run")
    });

    assert!(matches!(
        result,
        Err(Error::Io { path, .. }) if path == source.hardware_model()
    ));
    assert!(!target.exists());
}

#[test]
fn a_failed_copy_fails_with_the_target_path() {
    let directory = tempfile::tempdir().unwrap();
    let source = image(&directory.path().join("image"));
    let target = directory.path().join("vm");
    std::fs::create_dir(&target).unwrap();

    let result = clone_with(&source, &target, &seat_identity(), || {
        copy_directory(source.root(), &target)
    });

    assert!(matches!(
        result,
        Err(Error::Io { action: "clone the bundle", path, .. }) if path == target
    ));
}

/// The public `clone_bundle` with `clonefile`. The temporary folder of a Mac
/// is on APFS, so the macOS CI job runs this test. It starts no VM.
#[cfg(target_os = "macos")]
#[test]
fn clone_bundle_clones_an_image_without_identity_files_on_apfs() {
    let directory = tempfile::tempdir().unwrap();
    let image = image(&directory.path().join("image"));
    let target = directory.path().join("vm");
    let identity = seat_identity();

    let clone = crate::clone_bundle(&image, &target, &identity).unwrap();

    clone.check_files().unwrap();
    assert_eq!(clone.read_identity().unwrap(), identity);
    assert_eq!(clone.read_hardware_model().unwrap(), b"model");
    assert!(!image.machine_identifier().exists());
}

/// `clonefile` fails on an existing target, and the target stays as it was.
#[cfg(target_os = "macos")]
#[test]
fn clone_bundle_fails_on_an_existing_target_and_keeps_it() {
    let directory = tempfile::tempdir().unwrap();
    let source = image(&directory.path().join("image"));
    let existing = image(&directory.path().join("vm"));
    let other = Identity {
        mac_address: MacAddress::new([0xa2, 9, 9, 9, 9, 9]),
        machine_identifier: b"other".to_vec(),
    };
    existing.write_identity(&other).unwrap();

    let result = crate::clone_bundle(&source, existing.root(), &seat_identity());

    assert!(matches!(
        result,
        Err(Error::Io { action: "clone the bundle", path, .. }) if path == existing.root()
    ));
    assert_eq!(existing.read_identity().unwrap(), other);
}

/// A failed identity write removes the new clone, so a retry at the same
/// path can work. A folder at the source's machine identifier path makes
/// the write fail.
#[cfg(target_os = "macos")]
#[test]
fn clone_bundle_removes_the_clone_when_the_identity_write_fails() {
    let directory = tempfile::tempdir().unwrap();
    let image = image(&directory.path().join("image"));
    std::fs::create_dir(image.machine_identifier()).unwrap();
    let target = directory.path().join("vm");

    let result = crate::clone_bundle(&image, &target, &seat_identity());

    assert!(result.is_err());
    assert!(!target.exists());
}
