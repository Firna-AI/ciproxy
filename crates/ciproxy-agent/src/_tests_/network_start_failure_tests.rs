//! Failed starts retain cleanup intent and expose the original cleanup cause.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ciproxy_protocol::runners::RunnerState;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::CacheMock;
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, report_dependencies, spec, start_image};
use crate::host_tools::ToolsMock;
use crate::logging::subscriber;
use crate::network_barriers::Capture;
use crate::network_fixtures::{Deletion, HostNetworks, commands, restored, runtime, saving};
use crate::process::{CommandsMock, UnitState};
use crate::state::MachineState;
use crate::vm_files::VmFilesMock;
use crate::vsock::GuestsMock;

#[tokio::test]
async fn failed_start_logs_cleanup_cause_and_recovers_owned_resources_after_restart() {
    let host = Arc::new(Mutex::new(HostNetworks {
        fail_start: true,
        deletion: Deletion::Fail,
        ..HostNetworks::default()
    }));
    let saved = Arc::new(Mutex::new(Vec::new()));
    let stop_failed = Arc::new(AtomicBool::new(true));
    let stopping = stop_failed.clone();
    let mock = Arc::new(Unimock::new((
        commands(host.clone()),
        saving(saved.clone()),
        report_dependencies(),
        start_image(),
        ToolsMock::prepare.some_call(matching!()).returns(Ok(())),
        VmFilesMock::prepare.some_call(matching!(_)).returns(Ok(())),
        GuestsMock::detach
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        CommandsMock::unit
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                if stopping.load(Ordering::SeqCst) {
                    Err(Error::HostCommand {
                        program: "unit-inspection".to_owned(),
                    })
                } else {
                    Ok(UnitState::Absent)
                }
            })),
        VmFilesMock::remove.some_call(matching!(_)).returns(Ok(())),
        CacheMock::budget.some_call(matching!(_)).returns(Ok(())),
        CacheMock::refresh_budget
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
    )));
    let mut agent = controller(MachineState::default(), mock.as_ref().clone());
    agent.vms = Arc::new(runtime(mock.clone()));
    let capture = Capture::default();
    let writer = capture.clone();
    let logging = subscriber(BoxMakeWriter::new(move || writer.clone()), "warn");
    let result = agent.start(spec()).with_subscriber(logging).await;
    assert!(matches!(result, Err(Error::HostCommand { program }) if program == "systemd-run"));
    let state = restored(&saved);
    assert_eq!(
        state.failed_starts,
        [state.runners[0].local_id.clone()].into()
    );
    assert_eq!(state.runners[0].report.state, RunnerState::Stopping);
    assert_eq!(state.runners[0].network_index, 1);
    let output = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
    assert!(
        output.contains("[agent] a host tool failed: unit-inspection"),
        "cleanup cause must be logged: {output}"
    );
    stop_failed.store(false, Ordering::SeqCst);
    let mut recovered = controller(state, mock.as_ref().clone());
    recovered.vms = Arc::new(runtime(mock.clone()));
    recovered.restore().await.unwrap();
    let result = recovered.reconcile().await;
    assert!(matches!(result, Err(Error::HostCommand { program }) if program == "ip"));
    assert_eq!(recovered.state.lock().await.runners.len(), 0);
    assert_eq!(restored(&saved).failed_starts, Default::default());
    assert_eq!(restored(&saved).pending_networks, [1].into());
    let report = recovered.report().await.unwrap();
    assert_eq!(report.runners.len(), 0);
    host.lock().unwrap().deletion = Deletion::Succeed;
    recovered.reconcile().await.unwrap();
    assert_eq!(restored(&saved).pending_networks, Default::default());
    assert_eq!(host.lock().unwrap().namespaces, Default::default());
}
