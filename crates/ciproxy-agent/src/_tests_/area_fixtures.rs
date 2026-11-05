//! Captured effects make unsafe creation, formatting, and deletion observable.

use std::sync::{Arc, Mutex};

use unimock::{Clause, MockFn, Unimock, matching};

use crate::area::{AreaFilesMock, AreaMountsMock, PersistentArea};
use crate::area_types::{AreaRecord, Filesystem, Phase};
use crate::host_fixtures::record;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Event {
    Create(u64, u64),
    Format(String),
    Record(AreaRecord),
    Unit,
    Directories,
    Start,
    Reflinks,
    Stop(bool),
    Erase(AreaRecord),
    Reload,
}

pub(crate) fn area(mock: Arc<Unimock>) -> PersistentArea {
    PersistentArea {
        files: mock.clone(),
        mounts: mock,
    }
}

pub(crate) fn filesystem() -> Option<Filesystem> {
    Some(Filesystem {
        kind: "xfs".to_owned(),
        uuid: record().uuid,
    })
}

pub(crate) fn claim() -> impl Clause {
    AreaFilesMock::claim.some_call(matching!()).returns(Ok(()))
}

pub(crate) fn preparation(events: Arc<Mutex<Vec<Event>>>) -> impl Clause {
    let saved = events.clone();
    let unit = events.clone();
    let start = events.clone();
    let reflinks = events.clone();
    (
        AreaFilesMock::record
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, record| {
                saved.lock().unwrap().push(Event::Record(record.clone()));
                Ok(())
            })),
        AreaFilesMock::unit
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                unit.lock().unwrap().push(Event::Unit);
                Ok(())
            })),
        AreaMountsMock::start
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                start.lock().unwrap().push(Event::Start);
                Ok(())
            })),
        AreaMountsMock::reflinks
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                reflinks.lock().unwrap().push(Event::Reflinks);
                Ok(())
            })),
        AreaFilesMock::directories
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                events.lock().unwrap().push(Event::Directories);
                Ok(())
            })),
    )
}

pub(crate) fn creating(events: Arc<Mutex<Vec<Event>>>) -> impl Clause {
    AreaFilesMock::create
        .some_call(matching!(_, _))
        .answers_arc(Arc::new(move |_, host, bytes| {
            events.lock().unwrap().push(Event::Create(host, bytes));
            Ok(AreaRecord {
                phase: Phase::Creating,
                ..record()
            })
        }))
}

pub(crate) fn formatting(events: Arc<Mutex<Vec<Event>>>) -> impl Clause {
    AreaMountsMock::format
        .some_call(matching!(_))
        .answers_arc(Arc::new(move |_, record| {
            events
                .lock()
                .unwrap()
                .push(Event::Format(record.uuid.clone()));
            Ok(())
        }))
}

pub(crate) fn cleanup(events: Arc<Mutex<Vec<Event>>>) -> impl Clause {
    let unit = events.clone();
    let saved = events.clone();
    (
        AreaFilesMock::unit
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                unit.lock().unwrap().push(Event::Unit);
                Ok(())
            })),
        AreaFilesMock::record
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, record| {
                saved.lock().unwrap().push(Event::Record(record.clone()));
                Ok(())
            })),
        AreaMountsMock::stop
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, present| {
                events.lock().unwrap().push(Event::Stop(present));
                Ok(())
            })),
    )
}

pub(crate) fn erasing(events: Arc<Mutex<Vec<Event>>>) -> impl Clause {
    let erased = events.clone();
    (
        AreaFilesMock::erase
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, record| {
                erased.lock().unwrap().push(Event::Erase(record.clone()));
                Ok(())
            })),
        AreaMountsMock::reload
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                events.lock().unwrap().push(Event::Reload);
                Ok(())
            })),
    )
}
