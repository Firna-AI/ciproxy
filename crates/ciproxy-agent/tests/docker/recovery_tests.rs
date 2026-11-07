//! Failed attachments discard only safely closed data and keep reservations.

use std::fs;
use std::path::PathBuf;

use unimock::{MockFn, Unimock};

use crate::docker::DockerDisks;
use crate::docker_files::{self, AttachmentPhase};
use crate::error::Error;
use crate::fixtures::{TemporaryLayout, active_lease, vm};
use crate::process::CommandsMock;
use crate::state::VmRecord;

#[tokio::test]
async fn failed_writer_discards_source_and_keeps_capacity_through_restart_and_replay() {
    let temporary = TemporaryLayout::new();
    let record = vm("failed-writer");
    let repository = temporary.repository().await;
    let directory = temporary.layout.data.join("docker").join(&repository);
    fs::create_dir_all(&directory).unwrap();
    let base = directory.join("base.ext4");
    fs::write(&base, b"failed mount").unwrap();
    let path = temporary.layout.jail_root(&record).join("docker-job.ext4");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::hard_link(&base, &path).unwrap();
    docker_files::save(&temporary.layout, &record, &active_lease(repository, true)).unwrap();
    let adapter = temporary.adapter(Unimock::new(()));

    assert!(adapter.failed(&record).unwrap());
    assert!(adapter.pending(&record).unwrap());
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    let restored = temporary.adapter(Unimock::new(()));
    assert!(restored.discard(&record).await.is_err());
    assert!(!base.exists());
    assert!(restored.pending(&record).unwrap());
    assert_eq!(restored.report().unwrap().leased_bytes, 8192);
    fs::remove_dir(&path).unwrap();
    restored.discard(&record).await.unwrap();
    assert!(
        !base.exists(),
        "A failed writer disk must not serve the next job"
    );
    assert!(!path.exists());
    assert!(restored.reserved(&record).unwrap());
    assert_eq!(restored.report().unwrap().leased_bytes, 8192);
    let leases = docker_files::leases(&temporary.layout).unwrap();
    assert!(!leases[0].1.writer);
    assert!(leases[0].1.phase == AttachmentPhase::Detached);

    fs::write(&base, b"replacement").unwrap();
    assert!(!restored.failed(&record).unwrap());
    restored.discard(&record).await.unwrap();
    assert_eq!(fs::read(&base).unwrap(), b"replacement");
    restored.release(&record).unwrap();
    assert_eq!(restored.report().unwrap().leased_bytes, 0);
}

#[tokio::test]
async fn failed_vm_switch_keeps_source_even_when_no_writer_remains() {
    let temporary = TemporaryLayout::new();
    let record = vm("failed-switch");
    let repository = temporary.repository().await;
    let directory = temporary.layout.data.join("docker").join(&repository);
    fs::create_dir_all(&directory).unwrap();
    let base = directory.join("base.ext4");
    fs::write(&base, b"healthy source").unwrap();
    let path = temporary.layout.jail_root(&record).join("docker-job.ext4");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::hard_link(&base, &path).unwrap();
    let mut lease = active_lease(repository, true);
    lease.phase = AttachmentPhase::Preparing;
    docker_files::save(&temporary.layout, &record, &lease).unwrap();
    let adapter = temporary.adapter(Unimock::new(()));

    adapter.failed(&record).unwrap();
    adapter.discard(&record).await.unwrap();
    assert!(!path.exists(), "A closed VM attachment must be removed");
    assert_eq!(fs::read(&base).unwrap(), b"healthy source");
    assert_eq!(adapter.report().unwrap().leased_bytes, 8192);
}

#[tokio::test]
async fn failed_copy_preserves_healthy_idle_source_until_reservation_release() {
    let temporary = TemporaryLayout::new();
    let record = vm("healthy-idle-source");
    let (source, copy) = idle_source_and_failed_copy(&temporary, &record, b"healthy idle source");
    let expected_source = source.to_string_lossy().into_owned();
    let adapter = temporary.adapter(Unimock::new(
        CommandsMock::check_filesystem
            .some_call(&|check| {
                let expected_source = expected_source.clone();
                check.func(move |path, _| *path == expected_source);
            })
            .returns(Ok(())),
    ));

    assert!(adapter.failed(&record).unwrap());
    adapter.discard(&record).await.unwrap();
    assert!(
        source.exists(),
        "A healthy idle source must remain available"
    );
    assert_eq!(fs::read(&source).unwrap(), b"healthy idle source");
    assert!(!copy.exists());
    assert!(adapter.reserved(&record).unwrap());
    assert!(temporary.lease_path(&record).is_file());
    assert_eq!(adapter.report().unwrap().leased_bytes, 8192);

    adapter.release(&record).unwrap();
    assert!(!adapter.reserved(&record).unwrap());
    assert!(!temporary.lease_path(&record).exists());
    assert_eq!(adapter.report().unwrap().leased_bytes, 0);
}

#[tokio::test]
async fn failed_copy_discards_corrupt_idle_source_until_reservation_release() {
    let temporary = TemporaryLayout::new();
    let record = vm("corrupt-idle-source");
    let (source, copy) = idle_source_and_failed_copy(&temporary, &record, b"corrupt idle source");
    let expected_source = source.to_string_lossy().into_owned();
    let adapter = temporary.adapter(Unimock::new(
        CommandsMock::check_filesystem
            .some_call(&|check| {
                let expected_source = expected_source.clone();
                check.func(move |path, _| *path == expected_source);
            })
            .returns(Err(Error::DockerFilesystem)),
    ));

    assert!(adapter.failed(&record).unwrap());
    adapter.discard(&record).await.unwrap();
    assert!(!source.exists(), "A corrupt idle source must be discarded");
    assert!(!copy.exists());
    assert!(adapter.reserved(&record).unwrap());
    assert!(temporary.lease_path(&record).is_file());
    assert_eq!(adapter.report().unwrap().leased_bytes, 8192);

    adapter.release(&record).unwrap();
    assert!(!adapter.reserved(&record).unwrap());
    assert!(!temporary.lease_path(&record).exists());
    assert_eq!(adapter.report().unwrap().leased_bytes, 0);
}

fn idle_source_and_failed_copy(
    temporary: &TemporaryLayout,
    record: &VmRecord,
    source_bytes: &[u8],
) -> (PathBuf, PathBuf) {
    let repository = "repository".to_owned();
    let directory = temporary.layout.data.join("docker").join(&repository);
    fs::create_dir_all(&directory).unwrap();
    let source = directory.join("base.ext4");
    fs::write(&source, source_bytes).unwrap();
    let copy = temporary.layout.jail_root(record).join("docker-job.ext4");
    fs::create_dir_all(copy.parent().unwrap()).unwrap();
    fs::write(&copy, b"failed disposable copy").unwrap();
    docker_files::save(&temporary.layout, record, &active_lease(repository, false)).unwrap();
    (source, copy)
}
