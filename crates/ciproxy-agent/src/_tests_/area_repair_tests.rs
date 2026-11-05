//! Repeated install repairs setup without resizing, formatting, or hiding data.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::{AreaFilesMock, AreaMountsMock, DiskArea};
use crate::area_fixtures::{Event, area, claim, filesystem, preparation};
use crate::area_types::Phase;
use crate::error::Error;
use crate::host_fixtures::{inventory, record};

#[tokio::test]
async fn repair_preserves_a_ready_or_formatted_area_even_when_host_capacity_changes() {
    for (phase, old_minimum) in [
        (Phase::Ready, false),
        (Phase::Creating, false),
        (Phase::Ready, true),
    ] {
        let mut current = inventory();
        current.host_bytes = 500_000_000_000;
        current.record.as_mut().unwrap().phase = phase;
        let mut expected = record();
        if old_minimum {
            expected.source_capacity = 80_000_000_000;
            expected.image_budget = 4_000_000_000;
            expected.bytes = 66_000_003_072;
            expected.overhead = 2_000_003_072;
            current.host_bytes = 80_000_000_000;
            current.backing.as_mut().unwrap().bytes = expected.bytes;
            current.record = Some(expected.clone());
        }
        let events = Arc::new(Mutex::new(Vec::new()));
        let mock = Arc::new(Unimock::new((
            claim(),
            preparation(events.clone()),
            AreaFilesMock::inspect
                .some_call(matching!())
                .returns(Ok(current)),
            AreaMountsMock::filesystem
                .each_call(matching!())
                .answers(&|_| Ok(filesystem())),
            AreaMountsMock::mounted
                .some_call(matching!())
                .returns(Ok(true)),
        )));
        area(mock.clone()).prepare().await.unwrap();
        assert_eq!(
            *events.lock().unwrap(),
            vec![
                Event::Unit,
                Event::Start,
                Event::Reflinks,
                Event::Directories,
                Event::Record(expected),
            ],
            "{phase:?}, old_minimum={old_minimum}"
        );
    }
}

#[tokio::test]
async fn repair_refuses_foreign_missing_legacy_or_resized_backing_without_mutation() {
    let current = Arc::new(Mutex::new(inventory()));
    let loaded = current.clone();
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        AreaFilesMock::claim
            .each_call(matching!())
            .answers(&|_| Ok(())),
        preparation(events.clone()),
        AreaFilesMock::inspect
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(loaded.lock().unwrap().clone()))),
        AreaMountsMock::filesystem
            .each_call(matching!())
            .answers(&|_| Ok(filesystem())),
        AreaMountsMock::mounted
            .each_call(matching!())
            .answers(&|_| Ok(true)),
    )));
    area(mock.clone()).prepare().await.unwrap();
    assert_eq!(events.lock().unwrap().len(), 5);
    for problem in [
        "owner", "inode", "version", "size", "missing", "unowned", "legacy",
    ] {
        let mut invalid = inventory();
        match problem {
            "owner" => invalid.backing.as_mut().unwrap().safe = false,
            "inode" => invalid.backing.as_mut().unwrap().inode = 19,
            "version" => invalid.record.as_mut().unwrap().version = 3,
            "size" => invalid.backing.as_mut().unwrap().bytes = 90_000_000_000,
            "missing" => invalid.backing = None,
            "unowned" => invalid.record = None,
            _ => invalid.legacy = true,
        }
        *current.lock().unwrap() = invalid;
        let result = area(mock.clone()).prepare().await;
        assert!(
            matches!(result, Err(Error::Preparation { .. })),
            "{problem}"
        );
        assert_eq!(
            events.lock().unwrap().len(),
            5,
            "{problem} must not mutate data"
        );
    }
}

#[tokio::test]
async fn repair_and_verify_accept_host_remount_but_reject_foreign_xfs_identity() {
    let mut current = inventory();
    current.backing.as_mut().unwrap().device = 17;
    let observed = Arc::new(Mutex::new(filesystem()));
    let filesystem_now = observed.clone();
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        AreaFilesMock::claim
            .each_call(matching!())
            .answers(&|_| Ok(())),
        preparation(events.clone()),
        AreaFilesMock::inspect
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(current.clone()))),
        AreaMountsMock::filesystem
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                Ok(filesystem_now.lock().unwrap().clone())
            })),
        AreaMountsMock::mounted
            .each_call(matching!())
            .answers(&|_| Ok(true)),
    )));
    let repaired = area(mock.clone()).prepare().await;
    assert!(
        repaired.is_ok(),
        "the same owned XFS area must survive host device renumbering"
    );
    let verified = area(mock.clone()).verify().await;
    assert!(verified.is_ok());
    let mut refreshed = record();
    refreshed.device = 17;
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Unit,
            Event::Start,
            Event::Reflinks,
            Event::Directories,
            Event::Record(refreshed),
            Event::Reflinks,
        ]
    );
    observed.lock().unwrap().as_mut().unwrap().uuid = "foreign-filesystem".to_owned();
    let rejected = area(mock.clone()).prepare().await;
    assert!(matches!(
        rejected,
        Err(Error::Preparation {
            component: "XFS identity",
            ..
        })
    ));
    let rejected = area(mock.clone()).verify().await;
    assert!(matches!(
        rejected,
        Err(Error::Preparation {
            component: "XFS identity",
            ..
        })
    ));
    assert_eq!(events.lock().unwrap().len(), 6);
}
