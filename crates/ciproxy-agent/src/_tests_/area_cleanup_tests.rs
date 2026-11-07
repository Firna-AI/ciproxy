//! Uninstall commits removal intent and unmounts before deleting its backing file.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::{AreaFilesMock, AreaMountsMock, DiskArea};
use crate::area_fixtures::{Event, area, claim, cleanup, erasing, filesystem};
use crate::area_types::{AreaRecord, Phase};
use crate::error::Error;
use crate::host_fixtures::{inventory, record};

#[tokio::test]
async fn uninstall_erases_an_owned_creating_area_that_was_never_formatted() {
    let mut current = inventory();
    current.record.as_mut().unwrap().phase = Phase::Creating;
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        claim(),
        cleanup(events.clone()),
        erasing(events.clone()),
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Ok(current)),
        AreaMountsMock::filesystem
            .each_call(matching!())
            .answers(&|_| Ok(None)),
        AreaMountsMock::mounted
            .some_call(matching!())
            .returns(Ok(false)),
    )));
    let result = area(mock.clone()).remove().await;
    assert!(result.is_ok(), "owned blank Creating cleanup: {result:?}");
    let removing = AreaRecord {
        phase: Phase::Removing,
        ..record()
    };
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Unit,
            Event::Record(removing.clone()),
            Event::Stop(true),
            Event::Erase(removing),
            Event::Reload,
        ]
    );
}

#[tokio::test]
async fn uninstall_unmounts_before_erasing_only_its_recorded_area() {
    let mut current = inventory();
    current.backing.as_mut().unwrap().device = 17;
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        claim(),
        cleanup(events.clone()),
        erasing(events.clone()),
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Ok(current)),
        AreaMountsMock::filesystem
            .some_call(matching!())
            .returns(Ok(filesystem())),
        AreaMountsMock::mounted
            .some_call(matching!())
            .returns(Ok(false)),
    )));
    let result = area(mock.clone()).remove().await;
    assert!(
        result.is_ok(),
        "uninstall must accept the same filesystem after a host remount"
    );
    let removing = AreaRecord {
        device: 17,
        phase: Phase::Removing,
        ..record()
    };
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Unit,
            Event::Record(removing.clone()),
            Event::Stop(true),
            Event::Erase(removing),
            Event::Reload,
        ]
    );
}

#[tokio::test]
async fn a_still_mounted_area_is_preserved_after_an_unmount_attempt() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        claim(),
        cleanup(events.clone()),
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Ok(inventory())),
        AreaMountsMock::filesystem
            .some_call(matching!())
            .returns(Ok(filesystem())),
        AreaMountsMock::mounted
            .some_call(matching!())
            .returns(Ok(true)),
    )));
    let result = area(mock.clone()).remove().await;
    assert!(matches!(result, Err(Error::Preparation { .. })));
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Unit,
            Event::Record(AreaRecord {
                phase: Phase::Removing,
                ..record()
            }),
            Event::Stop(true),
        ]
    );
}

#[tokio::test]
async fn interrupted_uninstall_finishes_after_its_backing_file_was_removed() {
    let mut current = inventory();
    current.record.as_mut().unwrap().phase = Phase::Removing;
    current.backing = None;
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        claim(),
        cleanup(events.clone()),
        erasing(events.clone()),
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Ok(current)),
        AreaMountsMock::mounted
            .some_call(matching!())
            .returns(Ok(false)),
    )));
    let result = area(mock.clone()).remove().await;
    assert!(result.is_ok());
    let removing = AreaRecord {
        phase: Phase::Removing,
        ..record()
    };
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Unit,
            Event::Record(removing.clone()),
            Event::Stop(false),
            Event::Erase(removing),
            Event::Reload,
        ]
    );
}
