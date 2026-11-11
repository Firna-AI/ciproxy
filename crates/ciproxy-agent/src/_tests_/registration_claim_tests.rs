//! Delivery and retirement claims exclude each other before VM operations wait.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::{CommandId, RunnerId};
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_protocol::runners::RunnerState;
use tokio::sync::Notify;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{applied, controller, report_dependencies, spec, vm};
use crate::state::{MachineState, Mode};

#[path = "registration_barriers.rs"]
mod support;

use support::{HeldVms, Hold};

#[tokio::test]
async fn retirement_claim_rejects_a_retry_with_a_stale_starting_record() {
    let delivered = Arc::new(AtomicUsize::new(0));
    let sends = delivered.clone();
    let mock = Unimock::new((
        report_dependencies(),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::configuration_lost
            .some_call(matching!(_))
            .returns(Ok(true)),
        VmsMock::send
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, _| {
                sends.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
        VmsMock::stop.some_call(matching!(_)).returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
    ));
    let mut record = vm("runner-1", false);
    record.report.state = RunnerState::Starting;
    let saved = {
        let first = controller(
            MachineState {
                runners: vec![record],
                ..MachineState::default()
            },
            mock.clone(),
        );
        assert_eq!(
            first
                .control(ControlMessage::StartRunner {
                    command_id: CommandId("command-1".into()),
                    runner: spec(),
                })
                .await
                .unwrap(),
            applied()
        );
        serde_json::to_vec(&*first.state.lock().await).unwrap()
    };
    let delegate = Arc::new(mock.clone());
    let mut restored = controller(serde_json::from_slice(&saved).unwrap(), mock);
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    restored.vms = Arc::new(HeldVms {
        delegate,
        hold: Hold::Stop,
        started: started.clone(),
        release: release.clone(),
    });
    let agent = Arc::new(restored);
    agent.configuration_lost("runner-1").await.unwrap();
    let copied_starting = agent.state.lock().await.runners[0].clone();
    let removing = agent.clone();
    let removal = tokio::spawn(async move { removing.remove().await });
    started.notified().await;
    {
        let state = agent.state.lock().await;
        assert_eq!(state.mode, Mode::Removed);
        assert_eq!(state.runners[0].report.state, RunnerState::Stopping);
        assert!(state.runners[0].report.configuration_lost);
    }

    let retry = agent.retry_start(copied_starting, spec()).await;

    assert!(
        matches!(retry, Err(Error::RunnerMismatch)),
        "late retry must refuse delivery: {retry:?}"
    );
    assert_eq!(delivered.load(Ordering::SeqCst), 1);
    release.notify_one();
    removal.await.unwrap().unwrap();
    let report = agent.report().await.unwrap();
    assert_eq!(report.runners, vec![]);
    assert_eq!(report.slots[0].occupancy, SlotOccupancy::Free);
}

#[tokio::test]
async fn a_delivery_claim_keeps_concurrent_removal_on_guest_shutdown() {
    let mut record = vm("runner-1", false);
    record.report.state = RunnerState::Starting;
    record.report.configuration_lost = true;
    let copied_starting = record.clone();
    let mock = Unimock::new((
        report_dependencies(),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::configuration_lost.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Ok(true))
                .once()
                .then()
                .answers(&|_, _| Ok(false));
        }),
        VmsMock::request_stop
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::send.some_call(matching!(_, _)).returns(Ok(())),
    ));
    let delegate = Arc::new(mock.clone());
    let mut agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        mock,
    );
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    agent.vms = Arc::new(HeldVms {
        delegate,
        hold: Hold::Delivery,
        started: started.clone(),
        release: release.clone(),
    });
    let agent = Arc::new(agent);
    let delivering = agent.clone();
    let delivery =
        tokio::spawn(async move { delivering.retry_start(copied_starting, spec()).await });
    started.notified().await;

    agent.remove().await.unwrap();

    assert_eq!(
        agent.state.lock().await.runners[0].report.state,
        RunnerState::Stopping
    );
    assert!(
        !agent.state.lock().await.runners[0]
            .report
            .configuration_lost
    );
    release.notify_one();
    delivery.await.unwrap().unwrap();
    agent.remove().await.unwrap();
    let report = agent.report().await.unwrap();
    assert_eq!(report.runners.len(), 1);
    assert_eq!(report.runners[0].state, RunnerState::Stopping);
    assert!(!report.runners[0].configuration_lost);
    assert_eq!(
        report.slots[0].occupancy,
        SlotOccupancy::Warm {
            runner_id: RunnerId("runner-1".into())
        }
    );
}
