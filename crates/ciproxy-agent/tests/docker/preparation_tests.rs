//! Writer exclusion and disposable-copy failures use the production adapter.

use std::fs;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::CacheMock;
use crate::docker::DockerDisks;
use crate::docker_files::{self, Lease};
use crate::error::Error;
use crate::fixtures::{TemporaryLayout, facts, scope, vm};
use crate::process::{CommandsMock, arguments};

#[tokio::test]
async fn writer_lease_excludes_another_writer_until_release() {
    let temporary = TemporaryLayout::new();
    let owner = vm("owner");
    let next = vm("next");
    let repository = temporary.repository().await;
    docker_files::save(
        &temporary.layout,
        &owner,
        &Lease {
            repository,
            writer: true,
            bytes: 8192,
        },
    )
    .unwrap();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let captured = observed.clone();
    let path = temporary.lease_path(&next);
    let adapter = temporary.adapter(Unimock::new(
        CacheMock::budget
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                let lease: Lease = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                captured.lock().unwrap().push(lease.writer);
                Err(Error::CacheUnavailable)
            })),
    ));

    assert!(
        adapter
            .prepare(&next, &facts(), &scope(), 4096, 65536, 65536)
            .await
            .is_err()
    );
    assert_eq!(*observed.lock().unwrap(), [false]);
    adapter.release(&owner).unwrap();
    assert!(
        adapter
            .prepare(&next, &facts(), &scope(), 4096, 65536, 65536)
            .await
            .is_err()
    );
    assert_eq!(*observed.lock().unwrap(), [false, true]);
    assert!(!adapter.reserved(&next).unwrap());
    assert_eq!(adapter.report().unwrap().leased_bytes, 0);
}

#[tokio::test]
async fn disposable_copy_uses_valid_reflink_options_and_releases_failed_reservation() {
    let temporary = TemporaryLayout::new();
    let repository = temporary.repository().await;
    let vm = vm("copy");
    let directory = temporary.layout.data.join("docker").join(repository);
    fs::create_dir_all(&directory).unwrap();
    let base = directory.join("base.ext4");
    fs::write(&base, b"base").unwrap();
    let copy = temporary.layout.jail_root(&vm).join("docker-job.ext4");
    let captured = Arc::new(Mutex::new(Vec::new()));
    let observed = captured.clone();
    let adapter = temporary.adapter(Unimock::new((
        CacheMock::budget
            .some_call(matching!(_))
            .answers(&|_, _| Ok(())),
        CommandsMock::run
            .some_call(matching!("cp", _, []))
            .answers_arc(Arc::new(move |_, _, args, _| {
                *observed.lock().unwrap() = args.to_vec();
                Err(Error::HostCommand {
                    program: "cp".to_owned(),
                })
            })),
    )));
    let scope = JobScope::Unknown {
        reason: UnknownJobScopeReason::MissingToken,
    };

    let _ = adapter
        .prepare(&vm, &facts(), &scope, 4096, 65536, 65536)
        .await;

    assert_eq!(
        *captured.lock().unwrap(),
        arguments(&[
            "--reflink=always",
            "--sparse=auto",
            "--",
            &base.to_string_lossy(),
            &copy.to_string_lossy(),
        ])
    );
    assert!(!adapter.reserved(&vm).unwrap());
    assert_eq!(adapter.report().unwrap().leased_bytes, 0);
}
