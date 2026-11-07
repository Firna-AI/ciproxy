//! Captured uninstall effects expose routing and preparation order.

use std::sync::{Arc, Mutex};

use unimock::{Clause, MockFn, Unimock, matching};

use crate::boundaries::{ImagesMock, ServiceMock, StoreMock};
use crate::controller::AgentMock;
use crate::host_preparation::{FinalUninstall, PreparationMock};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Event {
    Verify,
    Recover,
    Restore,
    Drain,
    Area,
    Service,
    State,
}

pub(super) fn uninstall(mock: Arc<Unimock>) -> FinalUninstall {
    FinalUninstall {
        files: mock.clone(),
        images: mock.clone(),
        preparation: mock.clone(),
        service: mock.clone(),
        store: mock.clone(),
        agent: mock,
    }
}

pub(super) fn cleanup(events: Arc<Mutex<Vec<Event>>>) -> impl Clause {
    let area = events.clone();
    let service = events.clone();
    (
        AgentMock::cleanup_networks
            .some_call(matching!())
            .returns(Ok(())),
        PreparationMock::remove
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                area.lock().unwrap().push(Event::Area);
                Ok(())
            })),
        ServiceMock::uninstall
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                service.lock().unwrap().push(Event::Service);
                Ok(())
            })),
        StoreMock::clear
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                events.lock().unwrap().push(Event::State);
                Ok(())
            })),
    )
}

pub(super) fn runtime(events: Arc<Mutex<Vec<Event>>>) -> impl Clause {
    let verify = events.clone();
    let recover = events.clone();
    let restore = events.clone();
    (
        PreparationMock::verify
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                verify.lock().unwrap().push(Event::Verify);
                Ok(())
            })),
        ImagesMock::recover
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                recover.lock().unwrap().push(Event::Recover);
                Ok(())
            })),
        AgentMock::restore
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                restore.lock().unwrap().push(Event::Restore);
                Ok(())
            })),
        AgentMock::remove
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                events.lock().unwrap().push(Event::Drain);
                Ok(())
            })),
    )
}
