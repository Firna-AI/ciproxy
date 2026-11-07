//! Captured uninstall effects expose routing and preparation order.

use std::sync::{Arc, Mutex};

use unimock::{Clause, MockFn, Unimock, matching};

use crate::boundaries::ImagesMock;
use crate::controller::AgentMock;
use crate::host_preparation::{FinalUninstall, PreparationMock};
use crate::terminal_completion::{CompletionAction, UninstallCleanup};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Event {
    Verify,
    Recover,
    Restore,
    Drain,
    Complete,
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
    AgentMock::complete_removal
        .some_call(matching!(CompletionAction::Uninstall(UninstallCleanup {
            images: None,
            ..
        })))
        .answers_arc(Arc::new(move |_, _| {
            events.lock().unwrap().push(Event::Complete);
            Ok(true)
        }))
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
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                events.lock().unwrap().push(Event::Drain);
                Ok(())
            })),
    )
}
