//! Checked terminal cleanup preserves identity through each host cleanup failure.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::DiskAreaMock;
use crate::boundaries::{Images, ServiceMock, StoreMock, VmsMock};
use crate::error::{Error, ErrorResultExt, Result};
use crate::fixtures::{controller, identity};
use crate::host_preparation::{FinalUninstall, HostPreparation, Uninstall};
use crate::host_tools::ToolsMock;
use crate::images::ManagedImages;
use crate::network_fixtures::{restored, saving};
use crate::state::{MachineState, Mode};

use self::terminal_image_fixtures::storage;

#[path = "terminal_image_fixtures.rs"]
mod terminal_image_fixtures;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Failure {
    Area,
    UnmountedArea,
    Tools,
    Service,
    State,
}

#[derive(Debug, Eq, PartialEq)]
enum Outcome {
    Complete,
    Area,
    Tools,
    Service,
    State,
    Unexpected,
}

#[tokio::test]
async fn uninstall_completion_checks_networks_and_preserves_identity_through_cleanup_failures() {
    for (failure, expected, events_expected) in [
        (Failure::Area, Outcome::Area, vec!["network", "area"]),
        (
            Failure::UnmountedArea,
            Outcome::Area,
            vec!["network", "area"],
        ),
        (
            Failure::Tools,
            Outcome::Tools,
            vec!["network", "area", "tools"],
        ),
        (
            Failure::Service,
            Outcome::Service,
            vec!["network", "area", "tools", "service"],
        ),
        (
            Failure::State,
            Outcome::State,
            vec!["network", "area", "tools", "service", "identity"],
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
        let area = events.clone();
        let tools = events.clone();
        let service = events.clone();
        let cleared = events.clone();
        let bytes = saved.clone();
        let failing = Arc::new(AtomicBool::new(true));
        let area_failure = failing.clone();
        let tools_failure = failing.clone();
        let service_failure = failing.clone();
        let state_failure = failing.clone();
        let area_present = Arc::new(AtomicBool::new(true));
        let mounted = area_present.clone();
        let network_removed = Arc::new(AtomicBool::new(false));
        let inspected = network_removed.clone();
        let mock = Arc::new(Unimock::new((
            saving(saved.clone()),
            storage(area_present.clone(), events.clone()),
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
            DiskAreaMock::remove
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    area.lock().unwrap().push("area");
                    let failed = area_failure.load(Ordering::SeqCst)
                        && matches!(failure, Failure::Area | Failure::UnmountedArea);
                    if !failed || failure == Failure::UnmountedArea {
                        mounted.store(false, Ordering::SeqCst);
                    }
                    host_result(failed, "area")
                }))
                .at_least_times(0),
            ToolsMock::remove
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    tools.lock().unwrap().push("tools");
                    host_result(
                        tools_failure.load(Ordering::SeqCst) && failure == Failure::Tools,
                        "tools",
                    )
                }))
                .at_least_times(0),
            ServiceMock::uninstall
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    service.lock().unwrap().push("service");
                    host_result(
                        service_failure.load(Ordering::SeqCst) && failure == Failure::Service,
                        "service",
                    )
                }))
                .at_least_times(0),
            StoreMock::clear
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    cleared.lock().unwrap().push("identity");
                    if state_failure.load(Ordering::SeqCst) && failure == Failure::State {
                        return Err(io::Error::from(io::ErrorKind::PermissionDenied))
                            .context("fixture state cleanup denied");
                    }
                    *bytes.lock().unwrap() = serde_json::to_vec(&MachineState::default()).unwrap();
                    Ok(())
                }))
                .at_least_times(0),
        )));
        let images = Arc::new(ManagedImages::new(mock.clone(), mock.clone()));
        images.recover().unwrap();
        events.lock().unwrap().clear();
        let agent = Arc::new(controller(initial, mock.as_ref().clone()));
        let uninstall = FinalUninstall {
            files: mock.clone(),
            images,
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
            "failure: {failure:?}"
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
            Err(Error::Internal(_)) => Outcome::State,
            Err(_) => Outcome::Unexpected,
        };
        assert_eq!(outcome, expected, "failure: {failure:?}");
        assert_eq!(
            restored(&saved).identity,
            Some(identity()),
            "failure: {failure:?}"
        );
        assert_eq!(agent.state.lock().await.mode, Mode::Removed);
        assert_eq!(restored(&saved).pending_networks, Default::default());
        assert_eq!(restored(&saved).runners, vec![]);
        assert_eq!(
            area_present.load(Ordering::SeqCst),
            failure == Failure::Area
        );
        failing.store(false, Ordering::SeqCst);
        let repaired = uninstall.finish().await;
        assert_eq!(
            repaired
                .as_ref()
                .map(|_| ())
                .map_err(|error| error.to_string()),
            Ok(()),
            "repaired {failure:?} must finish in the same process: {repaired:?}"
        );
        let mut complete_events = events_expected;
        complete_events.extend(["area", "tools", "service", "identity"]);
        assert_eq!(
            *events.lock().unwrap(),
            complete_events,
            "repair: {failure:?}"
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
