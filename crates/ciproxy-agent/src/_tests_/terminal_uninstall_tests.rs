//! Checked terminal cleanup preserves identity through each host cleanup failure.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::DiskAreaMock;
use crate::boundaries::{ImagesMock, ServiceMock, StoreMock, VmsMock};
use crate::error::{Error, Result};
use crate::fixtures::{controller, identity};
use crate::host_preparation::{FinalUninstall, HostPreparation, Uninstall};
use crate::host_tools::ToolsMock;
use crate::network_fixtures::{restored, saving};
use crate::state::{MachineState, Mode};

#[derive(Debug, Eq, PartialEq)]
enum Outcome {
    Complete,
    Area,
    Tools,
    Service,
    Unexpected,
}

#[tokio::test]
async fn uninstall_completion_checks_networks_and_preserves_identity_through_cleanup_failures() {
    for (failure, expected, events_expected) in [
        ("area", Outcome::Area, vec!["network", "purge", "area"]),
        (
            "tools",
            Outcome::Tools,
            vec!["network", "purge", "area", "tools"],
        ),
        (
            "service",
            Outcome::Service,
            vec!["network", "purge", "area", "tools", "service"],
        ),
    ] {
        let initial = MachineState {
            mode: Mode::Removed,
            identity: Some(identity()),
            ..MachineState::default()
        };
        let saved = Arc::new(Mutex::new(serde_json::to_vec(&initial).unwrap()));
        let events = Arc::new(Mutex::new(Vec::new()));
        let network = events.clone();
        let purge = events.clone();
        let area = events.clone();
        let tools = events.clone();
        let service = events.clone();
        let cleared = events.clone();
        let bytes = saved.clone();
        let failing = Arc::new(AtomicBool::new(true));
        let area_failure = failing.clone();
        let tools_failure = failing.clone();
        let service_failure = failing.clone();
        let network_removed = Arc::new(AtomicBool::new(false));
        let inspected = network_removed.clone();
        let mock = Arc::new(Unimock::new((
            saving(saved.clone()),
            VmsMock::networks
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    Ok(if inspected.load(Ordering::SeqCst) {
                        vec![]
                    } else {
                        vec![1]
                    })
                }))
                .at_least_times(0),
            VmsMock::remove_network
                .some_call(matching!(1))
                .answers_arc(Arc::new(move |_, _| {
                    network.lock().unwrap().push("network");
                    network_removed.store(true, Ordering::SeqCst);
                    Ok(())
                }))
                .at_least_times(0),
            ImagesMock::purge
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    purge.lock().unwrap().push("purge");
                    Ok(())
                }))
                .at_least_times(0),
            DiskAreaMock::remove
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    area.lock().unwrap().push("area");
                    host_result(
                        area_failure.load(Ordering::SeqCst) && failure == "area",
                        "area",
                    )
                }))
                .at_least_times(0),
            ToolsMock::remove
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    tools.lock().unwrap().push("tools");
                    host_result(
                        tools_failure.load(Ordering::SeqCst) && failure == "tools",
                        "tools",
                    )
                }))
                .at_least_times(0),
            ServiceMock::uninstall
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    service.lock().unwrap().push("service");
                    host_result(
                        service_failure.load(Ordering::SeqCst) && failure == "service",
                        "service",
                    )
                }))
                .at_least_times(0),
            StoreMock::clear
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    cleared.lock().unwrap().push("identity");
                    *bytes.lock().unwrap() = serde_json::to_vec(&MachineState::default()).unwrap();
                    Ok(())
                }))
                .at_least_times(0),
        )));
        let agent = Arc::new(controller(initial, mock.as_ref().clone()));
        let uninstall = FinalUninstall {
            files: mock.clone(),
            images: mock.clone(),
            preparation: Arc::new(HostPreparation {
                files: mock.clone(),
                packages: mock.clone(),
                tools: mock.clone(),
                area: mock.clone(),
            }),
            service: mock.clone(),
            store: mock.clone(),
            agent: agent.clone(),
        };
        let result = uninstall.finish().await;
        assert_eq!(
            *events.lock().unwrap(),
            events_expected,
            "failure: {failure}"
        );
        let outcome = match result {
            Ok(()) => Outcome::Complete,
            Err(Error::Preparation {
                component: "XFS area cleanup",
                ..
            }) => Outcome::Area,
            Err(Error::Preparation {
                component: "owned Firecracker files",
                ..
            }) => Outcome::Tools,
            Err(Error::HostCommand { program }) if program == "service" => Outcome::Service,
            Err(_) => Outcome::Unexpected,
        };
        assert_eq!(outcome, expected, "failure: {failure}");
        assert_eq!(
            restored(&saved).identity,
            Some(identity()),
            "failure: {failure}"
        );
        assert_eq!(agent.state.lock().await.mode, Mode::Removed);
        failing.store(false, Ordering::SeqCst);
        uninstall.finish().await.unwrap();
        let mut complete_events = events_expected;
        complete_events.extend(["purge", "area", "tools", "service", "identity"]);
        assert_eq!(
            *events.lock().unwrap(),
            complete_events,
            "repair: {failure}"
        );
        assert_eq!(restored(&saved).identity, None);
        assert_eq!(agent.state.lock().await.mode, Mode::Removed);
    }
}

fn host_result(failure: bool, stage: &str) -> Result<()> {
    if failure {
        Err(Error::HostCommand {
            program: stage.to_owned(),
        })
    } else {
        Ok(())
    }
}
