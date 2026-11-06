//! Original free requests and actual allocations survive publication and restart.

use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::images::ImageReport;
use ciproxy_protocol::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, HostMock, ImagesMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::fixtures::{controller, hardware, lease, metadata, spec, vm};
use crate::reporting::empty_cache;
use crate::state::{MachineState, VmRecord};

#[tokio::test]
async fn retries_after_publication_and_restart_keep_the_original_request_and_allocation() {
    let selected = Arc::new(Mutex::new(metadata("image-1", 61_000_000_000)));
    let latest = selected.clone();
    let events = Arc::new(Mutex::new(Vec::new()));
    let held = events.clone();
    let allocated = events.clone();
    let saved = events.clone();
    let started = events.clone();
    let sent = events.clone();
    let persisted = Arc::new(Mutex::new(Vec::new()));
    let written = persisted.clone();
    let mock = Unimock::new((
        HostMock::hardware
            .each_call(matching!())
            .answers(&|_| Ok(hardware())),
        HostMock::hostname
            .each_call(matching!())
            .answers(&|_| Ok("machine".to_owned())),
        CacheMock::report
            .each_call(matching!())
            .answers(&|_| Ok(empty_cache())),
        CacheMock::budget
            .some_call(matching!(563_364_143_616))
            .returns(Ok(())),
        CacheMock::configure
            .each_call(matching!(_))
            .answers(&|_, _| Ok(None)),
        ImagesMock::report
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let metadata = latest.lock().unwrap().clone();
                Ok(ImageReport {
                    newest: Some(metadata.image_id.clone()),
                    available: vec![metadata],
                })
            })),
        ImagesMock::hold_newest
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                held.lock().unwrap().push("hold");
                Ok(lease(metadata("image-1", 61_000_000_000)))
            })),
        VmsMock::allocate
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, report, _| {
                allocated.lock().unwrap().push("allocate");
                Ok(VmRecord {
                    report,
                    ..vm("runner-1", false)
                })
            })),
        StoreMock::save
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, state| {
                saved.lock().unwrap().push("save");
                *written.lock().unwrap() = serde_json::to_vec(state).unwrap();
                Ok(())
            })),
        VmsMock::start
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, _| {
                started.lock().unwrap().push("start");
                Ok(())
            })),
        VmsMock::send
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, _| {
                sent.lock().unwrap().push("send");
                Ok(())
            })),
    ));
    let agent = controller(MachineState::default(), mock.clone());
    assert_eq!(
        agent
            .start(spec())
            .await
            .map_err(|error| discriminant(&error)),
        Ok(())
    );
    *selected.lock().unwrap() = metadata("image-2", 63_000_000_000);
    assert_eq!(
        agent
            .start(spec())
            .await
            .map_err(|error| discriminant(&error)),
        Ok(())
    );
    let restored: MachineState = serde_json::from_slice(&persisted.lock().unwrap()).unwrap();
    let restarted = controller(restored, mock.clone());
    assert_eq!(
        restarted
            .start(spec())
            .await
            .map_err(|error| discriminant(&error)),
        Ok(())
    );
    let report = restarted.report().await.unwrap();
    let mut expected = vm("runner-1", false).report;
    expected.state = RunnerState::Starting;
    expected.size.disk_bytes = 81_635_856_384;
    assert_eq!(report.runners, vec![expected]);
    assert_eq!(report.runners[0].requested_size.disk_bytes, 20_000_000_000);
    assert_eq!(report.runners[0].size.disk_bytes, 81_635_856_384);
    assert_eq!(
        report.images.selected(),
        Some(&metadata("image-2", 63_000_000_000))
    );
    assert_eq!(
        *events.lock().unwrap(),
        vec!["hold", "allocate", "save", "start", "send", "send"]
    );
}
