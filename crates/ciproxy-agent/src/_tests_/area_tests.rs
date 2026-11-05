//! Creation and interrupted first creation publish only a checked owned filesystem.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::{AreaFilesMock, AreaMountsMock, DiskArea};
use crate::area_fixtures::{Event, area, claim, creating, filesystem, formatting, preparation};
use crate::area_types::{Inventory, Phase};
use crate::host_fixtures::{inventory, record};

#[tokio::test]
async fn fresh_area_uses_job_disk_math_and_records_identity_before_mounting() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        claim(),
        preparation(events.clone()),
        formatting(events.clone()),
        creating(events.clone()),
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Ok(Inventory {
                record: None,
                backing: None,
                host_bytes: 80_000_000_000,
                free_bytes: 70_000_000_000,
                area_free: None,
                legacy: false,
            })),
        AreaMountsMock::filesystem
            .some_call(matching!())
            .returns(Ok(filesystem())),
        AreaMountsMock::mounted
            .some_call(matching!())
            .returns(Ok(true)),
    )));
    area(mock.clone()).prepare().await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Create(80_000_000_000, 60_000_000_000),
            Event::Format(record().uuid),
            Event::Record(record()),
            Event::Unit,
            Event::Start,
            Event::Reflinks,
            Event::Directories,
        ]
    );
}

#[tokio::test]
async fn interrupted_creation_formats_only_its_unpublished_empty_area() {
    let mut current = inventory();
    current.record.as_mut().unwrap().phase = Phase::Creating;
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        claim(),
        preparation(events.clone()),
        formatting(events.clone()),
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Ok(current)),
        AreaMountsMock::filesystem.stub(|each| {
            each.call(matching!())
                .answers(&|_| Ok(None))
                .once()
                .then()
                .answers(&|_| Ok(filesystem()));
        }),
        AreaMountsMock::mounted
            .some_call(matching!())
            .returns(Ok(true)),
    )));
    area(mock.clone()).prepare().await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Format(record().uuid),
            Event::Record(record()),
            Event::Unit,
            Event::Start,
            Event::Reflinks,
            Event::Directories,
        ]
    );
}
