//! Interrupted cleanup and publication keep the Docker lease ledger recoverable.

use std::fs;

use unimock::Unimock;

use crate::docker::DockerDisks;
use crate::docker_files::{self, Lease};
use crate::fixtures::{TemporaryLayout, vm};

#[test]
fn interrupted_writer_cleanup_releases_reservation() {
    let temporary = TemporaryLayout::new();
    let vm = vm("retired-writer");
    docker_files::save(
        &temporary.layout,
        &vm,
        &Lease {
            repository: "repository".to_owned(),
            writer: true,
            bytes: 8192,
            phase: docker_files::AttachmentPhase::Offered,
        },
    )
    .unwrap();
    let adapter = temporary.adapter(Unimock::new(()));

    assert!(adapter.release(&vm).is_ok());
    assert!(!adapter.reserved(&vm).unwrap());
    assert_eq!(adapter.report().unwrap().leased_bytes, 0);
    assert!(adapter.release(&vm).is_ok());
}

#[test]
fn cleanup_failure_keeps_reservation_until_file_removal_succeeds() {
    let temporary = TemporaryLayout::new();
    let vm = vm("failed-cleanup");
    docker_files::save(
        &temporary.layout,
        &vm,
        &Lease {
            repository: "repository".to_owned(),
            writer: false,
            bytes: 8192,
            phase: docker_files::AttachmentPhase::Offered,
        },
    )
    .unwrap();
    let path = temporary.layout.jail_root(&vm).join("docker-job.ext4");
    fs::create_dir_all(&path).unwrap();
    let adapter = temporary.adapter(Unimock::new(()));

    assert!(adapter.release(&vm).is_err());
    assert!(adapter.reserved(&vm).unwrap());
    assert_eq!(adapter.report().unwrap().leased_bytes, 8192);
    fs::remove_dir(path).unwrap();
    adapter.release(&vm).unwrap();
    assert!(!adapter.reserved(&vm).unwrap());
}

#[test]
fn release_does_not_decode_another_vm_lease() {
    let temporary = TemporaryLayout::new();
    let vm = vm("independent-cleanup");
    docker_files::save(
        &temporary.layout,
        &vm,
        &Lease {
            repository: "repository".to_owned(),
            writer: false,
            bytes: 8192,
            phase: docker_files::AttachmentPhase::Offered,
        },
    )
    .unwrap();
    let unrelated = temporary.layout.data.join("docker-leases/local-other");
    fs::write(&unrelated, b"interrupted unrelated data").unwrap();
    let adapter = temporary.adapter(Unimock::new(()));

    assert!(adapter.release(&vm).is_ok());
    assert!(!temporary.lease_path(&vm).exists());
    assert_eq!(fs::read(unrelated).unwrap(), b"interrupted unrelated data");
}

#[test]
fn complete_pending_lease_does_not_hold_capacity() {
    pending_lease_does_not_hold_capacity(
        br#"{"repository":"repository","writer":true,"bytes":8192}"#,
    );
}

#[test]
fn partial_pending_lease_does_not_block_reporting() {
    pending_lease_does_not_hold_capacity(b"");
}

fn pending_lease_does_not_hold_capacity(bytes: &[u8]) {
    let temporary = TemporaryLayout::new();
    let vm = vm("published");
    docker_files::save(
        &temporary.layout,
        &vm,
        &Lease {
            repository: "repository".to_owned(),
            writer: true,
            bytes: 8192,
            phase: docker_files::AttachmentPhase::Offered,
        },
    )
    .unwrap();
    let pending = temporary.lease_path(&vm).with_extension("pending");
    fs::write(&pending, bytes).unwrap();
    let adapter = temporary.adapter(Unimock::new(()));

    let report = adapter.report();
    assert!(report.is_ok());
    assert_eq!(report.unwrap().leased_bytes, 8192);
    assert!(adapter.reserved(&vm).unwrap());
    assert_eq!(fs::read(pending).unwrap(), bytes);
}
