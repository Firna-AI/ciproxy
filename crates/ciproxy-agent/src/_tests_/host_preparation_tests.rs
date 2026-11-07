//! Host preparation and final uninstall preserve dependency and cleanup order.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::{AreaFilesMock, DiskAreaMock};
use crate::area_types::Phase;
use crate::boundaries::{ImagesMock, ServiceMock, StoreMock, VmsMock};
use crate::error::{Error, ErrorResultExt};
use crate::fixtures::{controller, vm};
use crate::host_fixtures::inventory;
use crate::host_packages::PackagesMock;
use crate::host_preparation::{
    FinalUninstall, HostPreparation, Preparation, PreparationMock, Uninstall, UninstallState,
};
use crate::host_tools::ToolsMock;
use crate::state::{MachineState, Mode};

use super::uninstall_fixtures::{Event, cleanup, runtime, uninstall};

#[tokio::test]
async fn uninstall_cleans_an_empty_creating_machine_and_verifies_a_recorded_vm_before_drain() {
    let mut current = inventory();
    current.record.as_mut().unwrap().phase = Phase::Creating;
    let state = MachineState {
        runners: vec![vm("running", true)],
        ..MachineState::default()
    };
    let events = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        AreaFilesMock::inspect.stub(|each| {
            each.call(matching!())
                .answers_arc(Arc::new(move |_| Ok(current.clone())))
                .once()
                .then()
                .answers(&|_| Ok(inventory()));
        }),
        StoreMock::load.stub(|each| {
            each.call(matching!())
                .answers(&|_| Ok(MachineState::default()))
                .once()
                .then()
                .answers_arc(Arc::new(move |_| Ok(state.clone())));
        }),
        runtime(events.clone()),
        cleanup(events.clone()),
    )));
    let service = uninstall(mock.clone());
    let result = service.start().await;
    assert_eq!(
        *events.lock().unwrap(),
        vec![Event::Drain, Event::Complete],
        "an empty Creating machine must reach guarded removal without runtime checks"
    );
    assert_eq!(result.unwrap(), UninstallState::Complete);
    let result = service.start().await;
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            Event::Drain,
            Event::Complete,
            Event::Verify,
            Event::Recover,
            Event::Restore,
            Event::Drain,
        ],
        "a recorded VM must verify before recovery, restoration, and drain"
    );
    assert_eq!(result.unwrap(), UninstallState::Draining);
}

#[tokio::test]
async fn uninstall_keeps_the_area_ownership_repair_message_when_inspection_fails() {
    let failure = Err::<(), _>(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
        .context("fixture ownership inspection denied")
        .unwrap_err();
    let mock = Arc::new(Unimock::new(
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Err(failure)),
    ));
    let error = uninstall(mock).start().await.unwrap_err();
    assert_eq!(
        error.to_string(),
        "[agent/host] area ownership record: Restore area.json and its matching backing file from backup. Then rerun uninstall."
    );
}

#[tokio::test]
async fn preparation_secures_control_before_packages_tools_and_the_area() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let claimed = events.clone();
    let packages = events.clone();
    let tools = events.clone();
    let area = events.clone();
    let mock = Arc::new(Unimock::new((
        AreaFilesMock::claim
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                claimed.lock().unwrap().push("control");
                Ok(())
            }))
            .at_least_times(0),
        PackagesMock::prepare
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                packages.lock().unwrap().push("packages");
                Ok(())
            }))
            .at_least_times(0),
        ToolsMock::prepare
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                tools.lock().unwrap().push("tools");
                Ok(())
            }))
            .at_least_times(0),
        DiskAreaMock::prepare
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                area.lock().unwrap().push("area");
                Ok(())
            }))
            .at_least_times(0),
    )));
    HostPreparation {
        files: mock.clone(),
        packages: mock.clone(),
        tools: mock.clone(),
        area: mock.clone(),
    }
    .prepare()
    .await
    .unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec!["control", "packages", "tools", "area"]
    );
}

#[tokio::test]
async fn final_uninstall_waits_for_jobs_then_preserves_identity_until_cleanup_succeeds() {
    for jobs_finished in [false, true] {
        let events = Arc::new(Mutex::new(Vec::new()));
        let purged = events.clone();
        let removed = events.clone();
        let uninstalled = events.clone();
        let cleared = events.clone();
        let callbacks = (
            ImagesMock::purge
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    purged.lock().unwrap().push("purge");
                    Ok(())
                }))
                .at_least_times(0),
            PreparationMock::remove
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    removed.lock().unwrap().push("area");
                    Ok(())
                }))
                .at_least_times(0),
            ServiceMock::uninstall
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    uninstalled.lock().unwrap().push("service");
                    Ok(())
                }))
                .at_least_times(0),
            StoreMock::clear
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    cleared.lock().unwrap().push("identity");
                    Ok(())
                }))
                .at_least_times(0),
        );
        let network = VmsMock::networks
            .some_call(matching!())
            .answers(&|_| Ok(Vec::new()))
            .at_least_times(0);
        let mock = Arc::new(if jobs_finished {
            Unimock::new((network, callbacks))
        } else {
            Unimock::new(())
        });
        let agent = controller(
            MachineState {
                mode: Mode::Removed,
                runners: if jobs_finished {
                    vec![]
                } else {
                    vec![vm("job", true)]
                },
                ..MachineState::default()
            },
            mock.as_ref().clone(),
        );
        let result = FinalUninstall {
            files: mock.clone(),
            images: mock.clone(),
            preparation: mock.clone(),
            service: mock.clone(),
            store: mock.clone(),
            agent: Arc::new(agent),
        }
        .finish()
        .await;
        let expected = if jobs_finished {
            vec!["purge", "area", "service", "identity"]
        } else {
            vec![]
        };
        assert_eq!(*events.lock().unwrap(), expected);
        assert_eq!(matches!(result, Err(Error::Draining)), !jobs_finished);
        assert_eq!(result.is_ok(), jobs_finished);
    }
}

#[tokio::test]
async fn denied_backing_creation_keeps_an_actionable_error_at_the_live_entrypoint() {
    let failure = Err::<(), _>(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
        .context("fixture backing creation denied");
    let mock = Arc::new(Unimock::new((
        AreaFilesMock::claim.some_call(matching!()).returns(Ok(())),
        PackagesMock::prepare.some_call(matching!()).returns(Ok(())),
        ToolsMock::prepare.some_call(matching!()).returns(Ok(())),
        DiskAreaMock::prepare
            .some_call(matching!())
            .returns(failure),
    )));
    let error = HostPreparation {
        files: mock.clone(),
        packages: mock.clone(),
        tools: mock.clone(),
        area: mock.clone(),
    }
    .prepare()
    .await
    .unwrap_err();
    assert!(matches!(
        &error,
        Error::Preparation {
            component: "XFS area",
            ..
        }
    ));
    assert!(error.to_string().contains("/var/lib/ciproxy/area.xfs"));
    assert!(error.to_string().contains("permissions and free space"));
}

#[tokio::test]
async fn area_verification_and_uninstall_do_not_require_the_current_runtime_pin() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let verified = events.clone();
    let area = events.clone();
    let tools = events.clone();
    let mock = Arc::new(Unimock::new((
        DiskAreaMock::verify
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                verified.lock().unwrap().push("verify-area");
                Ok(())
            })),
        DiskAreaMock::remove
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                area.lock().unwrap().push("remove-area");
                Ok(())
            })),
        ToolsMock::remove
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                tools.lock().unwrap().push("remove-tools");
                Ok(())
            })),
    )));
    let preparation = HostPreparation {
        files: mock.clone(),
        packages: mock.clone(),
        tools: mock.clone(),
        area: mock.clone(),
    };
    let verified = preparation.verify().await;
    assert!(verified.is_ok());
    let removed = preparation.remove().await;
    assert!(removed.is_ok());
    assert_eq!(
        *events.lock().unwrap(),
        vec!["verify-area", "remove-area", "remove-tools"]
    );
}
