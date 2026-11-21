//! Published sizing keeps the complete job disk and resumes only recorded growth.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::{AreaFilesMock, AreaMountsMock, DiskArea, PersistentArea};
use crate::area_fixtures::{Event, area, claim, filesystem, preparation};
use crate::area_types::{AreaRecord, Phase};
use crate::error::Error;
use crate::host_fixtures::{inventory, record};

#[tokio::test]
async fn legacy_or_changed_budget_cannot_publish_the_old_small_area() {
    for (version, requested, expected) in [
        (1, None, "old workload area format"),
        (2, Some(8_000_000_000), "image budget changed"),
    ] {
        let current = Arc::new(Mutex::new(inventory()));
        let observed = current.clone();
        let events = Arc::new(Mutex::new(Vec::new()));
        let mock = Arc::new(Unimock::new((
            AreaFilesMock::claim
                .each_call(matching!())
                .answers(&|_| Ok(())),
            preparation(events.clone()),
            AreaFilesMock::inspect
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| Ok(observed.lock().unwrap().clone()))),
            AreaMountsMock::filesystem
                .each_call(matching!())
                .answers(&|_| Ok(filesystem())),
            AreaMountsMock::mounted
                .each_call(matching!())
                .answers(&|_| Ok(true)),
        )));
        area(mock.clone()).prepare().await.unwrap();
        current.lock().unwrap().record.as_mut().unwrap().version = version;
        let area = PersistentArea {
            files: mock.clone(),
            mounts: mock.clone(),
            requested_budget: requested,
        };
        let result = area.prepare().await;
        assert!(
            matches!(result, Err(Error::Preparation { component, .. }) if component == expected)
        );
        assert_eq!(events.lock().unwrap().len(), 5);
    }
}

#[tokio::test]
async fn recorded_unpublished_growth_resumes_before_ready_without_reformatting() {
    let mut current = inventory();
    let mut growing = record();
    growing.phase = Phase::Creating;
    growing.growth_from = Some(222_500_003_840);
    current.record = Some(growing.clone());
    current.backing.as_mut().unwrap().bytes = 222_500_003_840;
    let events = Arc::new(Mutex::new(Vec::new()));
    let files = events.clone();
    let grown = growing.clone();
    let mock = Arc::new(Unimock::new((
        claim(), preparation(events.clone()),
        AreaFilesMock::inspect.some_call(matching!()).returns(Ok(current)),
        AreaMountsMock::filesystem.each_call(matching!()).answers(&|_| Ok(filesystem())),
        AreaMountsMock::mounted.some_call(matching!()).returns(Ok(true)),
        AreaFilesMock::grow.some_call(matching!((record, bytes) if record.growth_from == Some(222_500_003_840) && *bytes == 226_000_003_072))
            .answers_arc(Arc::new(move |_, _, _| { files.lock().unwrap().push(Event::Grow); Ok(grown.clone()) })),
        AreaMountsMock::grow.some_call(matching!()).returns(Ok(())),
    )));
    let result = area(mock.clone()).prepare().await;
    assert!(result.is_ok());
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Grow,
            Event::Unit,
            Event::Start,
            Event::Reflinks,
            Event::Record(AreaRecord {
                phase: Phase::Creating,
                ..record()
            }),
            Event::Directories,
            Event::Record(record()),
        ]
    );
}

#[tokio::test]
async fn fresh_growth_measures_actual_overhead_before_publishing_ready() {
    let mut current = inventory();
    let mut starting = record();
    starting.phase = Phase::Creating;
    starting.bytes = 222_500_003_840;
    starting.overhead = 0;
    current.record = Some(starting.clone());
    current.backing.as_mut().unwrap().bytes = starting.bytes;
    let captured = Arc::new(Mutex::new(Vec::new()));
    let observations = captured.clone();
    let grown = Arc::new(Mutex::new(Vec::new()));
    let sizes = grown.clone();
    let mock = Arc::new(Unimock::new((
        claim(),
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Ok(current)),
        AreaMountsMock::filesystem
            .each_call(matching!())
            .answers(&|_| Ok(filesystem())),
        AreaFilesMock::unit.some_call(matching!()).returns(Ok(())),
        AreaMountsMock::start.some_call(matching!()).returns(Ok(())),
        AreaMountsMock::mounted
            .some_call(matching!())
            .returns(Ok(true)),
        AreaMountsMock::reflinks
            .some_call(matching!())
            .returns(Ok(())),
        AreaFilesMock::directories
            .some_call(matching!())
            .returns(Ok(())),
        AreaMountsMock::available.stub(|each| {
            each.call(matching!())
                .answers(&|_| Ok(222_400_003_840))
                .once()
                .then()
                .answers(&|_| Ok(225_094_905_600));
        }),
        AreaFilesMock::grow
            .some_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, record, bytes| {
                sizes.lock().unwrap().push(bytes);
                Ok(AreaRecord {
                    bytes,
                    growth_from: Some(record.bytes),
                    ..record.clone()
                })
            })),
        AreaMountsMock::grow.some_call(matching!()).returns(Ok(())),
        AreaFilesMock::record
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, record| {
                observations.lock().unwrap().push(record.clone());
                Ok(())
            })),
    )));
    let result = area(mock.clone()).prepare().await;
    assert!(result.is_ok());
    assert_eq!(*grown.lock().unwrap(), vec![225_194_905_600]);
    let saved = captured.lock().unwrap();
    assert_eq!(saved.last().unwrap().phase, Phase::Ready);
    assert_eq!(saved.last().unwrap().image_budget, 140_000_000_000);
    assert_eq!(saved.last().unwrap().overhead, 2_694_905_600);
    assert_eq!(saved.last().unwrap().growth_from, None);
}
