//! Admission uses the release held after the preceding machine report.

use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::runners::{RunnerReport, RunnerState};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, ImagesMock, StoreMock, VmsMock};
use crate::error::Error;
use crate::fixtures::{controller, hardware, metadata, report_dependencies_for, spec, vm};
use crate::image_registry::{ImageLease, ImageLeaseMock};
use crate::state::{MachineState, VmRecord};

#[derive(Debug, Eq, PartialEq)]
enum Event {
    Saved(RunnerReport),
    Budget(u64),
    Started(RunnerReport),
}

#[tokio::test]
async fn publication_after_report_rechecks_capacity_and_allocates_the_held_release() {
    for (capacity, expected_error, expected_events) in [
        (252_000_000_000, None, vec!["saved", "budget", "started"]),
        (
            250_000_000_000,
            Some(discriminant(&Error::Capacity)),
            vec![],
        ),
    ] {
        let events = Arc::new(Mutex::new(Vec::new()));
        let saved = events.clone();
        let budgeted = events.clone();
        let started = events.clone();
        let mut machine = hardware();
        machine.disks[0].capacity_bytes = capacity;
        let held: Arc<dyn ImageLease> = Arc::new(Unimock::new(
            ImageLeaseMock::metadata
                .some_call(matching!())
                .returns(metadata("newer", 63_000_000_000)),
        ));
        let dependencies = (
            report_dependencies_for(machine),
            ImagesMock::hold_newest
                .some_call(matching!())
                .returns(Ok(held)),
        );
        let mock = if expected_error.is_some() {
            Unimock::new(dependencies)
        } else {
            Unimock::new((
                dependencies,
                VmsMock::allocate
                    .each_call(matching!(_, _))
                    .answers(&|_, report, _| {
                        Ok(VmRecord {
                            report,
                            ..vm("new", false)
                        })
                    })
                    .at_least_times(0),
                StoreMock::save
                    .each_call(matching!(_))
                    .answers_arc(Arc::new(move |_, state| {
                        saved
                            .lock()
                            .unwrap()
                            .push(Event::Saved(state.runners[0].report.clone()));
                        Ok(())
                    }))
                    .at_least_times(0),
                CacheMock::budget
                    .each_call(matching!(_))
                    .answers_arc(Arc::new(move |_, bytes| {
                        budgeted.lock().unwrap().push(Event::Budget(bytes));
                        Ok(())
                    }))
                    .at_least_times(0),
                VmsMock::start
                    .each_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, record, _| {
                        started
                            .lock()
                            .unwrap()
                            .push(Event::Started(record.report.clone()));
                        Ok(())
                    }))
                    .at_least_times(0),
            ))
        };
        let agent = controller(MachineState::default(), mock);
        assert_eq!(
            agent
                .start(spec())
                .await
                .err()
                .map(|error| discriminant(&error)),
            expected_error
        );
        let observed = events.lock().unwrap();
        let names: Vec<_> = observed
            .iter()
            .map(|event| match event {
                Event::Saved(_) => "saved",
                Event::Budget(_) => "budget",
                Event::Started(_) => "started",
            })
            .collect();
        assert_eq!(names, expected_events);
        let mut expected = vm("runner-1", false).report;
        expected.image_id = metadata("newer", 63_000_000_000).image_id;
        expected.size.disk_bytes = 83_635_855_360;
        expected.state = RunnerState::Starting;
        let expected_full = expected_events
            .first()
            .map(|_| {
                vec![
                    Event::Saved(expected.clone()),
                    Event::Budget(364_144_640),
                    Event::Started(expected),
                ]
            })
            .unwrap_or_default();
        assert_eq!(*observed, expected_full);
    }
}
