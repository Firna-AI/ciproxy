//! Every reclaimer preserves an in-progress source until its copy owns blocks.

use std::fs::{self, File};
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::CacheMock;
use crate::docker::{DockerDisks, LinuxDockers};
use crate::error::Error;
use crate::fixtures::{TemporaryLayout, facts, vm};
use crate::process::CommandsMock;
use crate::vm_drives::VmDrive;

#[tokio::test]
async fn a_selected_copy_source_stays_held_across_reclaim_then_releases_before_growth() {
    let temporary = TemporaryLayout::new();
    let repository = temporary.repository().await;
    let base = temporary.base(&repository);
    fs::create_dir_all(base.parent().unwrap()).unwrap();
    fs::write(&base, vec![b'B'; 8192]).unwrap();
    let record = vm("held-copy");
    fs::create_dir_all(temporary.layout.jail_root(&record)).unwrap();
    let path = VmDrive::DockerJob.path(&temporary.layout, &record);
    let reclaimer = LinuxDockers {
        layout: temporary.layout.clone(),
        commands: Arc::new(Unimock::new(())),
        allocation: Arc::new(tokio::sync::Mutex::new(())),
        space: Arc::new(Unimock::new(())),
        image_budget: 0,
    };
    let source = base.clone();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let budgets = observed.clone();
    let initial_source = base.clone();
    let mock = Arc::new(Unimock::new((
        CacheMock::budget
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                budgets
                    .lock()
                    .unwrap()
                    .push((bytes, initial_source.exists()));
                Ok(())
            })),
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, program, _, _| {
                if program == "cp" {
                    assert_eq!(reclaimer.reclaim(0).unwrap(), 8192);
                    assert!(
                        source.exists(),
                        "another grant must preserve a selected source until cp completes"
                    );
                    fs::copy(&source, &path).unwrap();
                }
                Ok(b"Free blocks: 1\nBlock size: 4096\n".to_vec())
            })),
        CommandsMock::check_filesystem
            .some_call(matching!(_))
            .returns(Err(Error::HostCommand {
                program: "source-stop".to_owned(),
            })),
    )));
    let adapter = LinuxDockers {
        layout: temporary.layout.clone(),
        commands: mock.clone(),
        allocation: Arc::new(tokio::sync::Mutex::new(())),
        space: temporary.space(mock, 8192),
        image_budget: 0,
    };
    let scope = JobScope::Unknown {
        reason: UnknownJobScopeReason::MissingToken,
    };
    let result = adapter.prepare(&record, &facts(), &scope, 4096, 8192).await;
    assert!(matches!(result, Err(Error::HostCommand { program }) if program == "source-stop"));
    assert_eq!(*observed.lock().unwrap(), vec![(0, true), (0, false)]);
    assert!(!base.exists());
    assert_eq!(adapter.report().unwrap().leased_bytes, 0);
}

#[tokio::test]
async fn copying_a_repository_refreshes_its_lru_use_before_the_next_reclaim() {
    let temporary = TemporaryLayout::new();
    let repository = temporary.repository().await;
    let base = temporary.base(&repository);
    let idle = temporary.base("newer-idle");
    for (path, age) in [(&base, 1), (&idle, 2)] {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![b'B'; 8192]).unwrap();
        File::open(path)
            .unwrap()
            .set_modified(UNIX_EPOCH + Duration::from_secs(age))
            .unwrap();
    }
    let record = vm("refresh-copy");
    fs::create_dir_all(temporary.layout.jail_root(&record)).unwrap();
    let copy = VmDrive::DockerJob.path(&temporary.layout, &record);
    let source = base.clone();
    let reclaimer = LinuxDockers {
        layout: temporary.layout.clone(),
        commands: Arc::new(Unimock::new(())),
        allocation: Arc::new(tokio::sync::Mutex::new(())),
        space: Arc::new(Unimock::new(())),
        image_budget: 0,
    };
    let adapter = temporary.adapter(Unimock::new((
        CacheMock::budget
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        CommandsMock::run
            .some_call(matching!("cp", _, _))
            .answers_arc(Arc::new(move |_, _, _, _| {
                fs::copy(&source, &copy).unwrap();
                Ok(Vec::new())
            })),
        CommandsMock::check_filesystem
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                assert_eq!(reclaimer.reclaim(8192).unwrap(), 8192);
                Err(Error::HostCommand {
                    program: "recency-stop".to_owned(),
                })
            })),
    )));
    let scope = JobScope::Unknown {
        reason: UnknownJobScopeReason::MissingToken,
    };
    let result = adapter
        .prepare(&record, &facts(), &scope, 4096, 65536)
        .await;
    assert!(matches!(result, Err(Error::HostCommand { program }) if program == "recency-stop"));
    assert!(
        base.exists(),
        "the most recently copied repository must stay cached"
    );
    assert!(!idle.exists());
}
