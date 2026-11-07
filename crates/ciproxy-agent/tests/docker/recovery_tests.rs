//! Failed attachments discard only safely closed data and keep reservations.

use std::fs;

use unimock::Unimock;

use crate::docker::DockerDisks;
use crate::docker_files::{self, AttachmentPhase};
use crate::fixtures::{TemporaryLayout, active_lease, vm};

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
