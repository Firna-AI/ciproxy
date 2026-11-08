//! The production daemon preserves independent reports and terminal revocation.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::task::Poll;

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use futures_util::{pin_mut, poll};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::unconstrained;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{ConnectionMock, DelayMock};
use crate::connection_types::ControlEvent;
use crate::controller::AgentMock;
use crate::fixtures::{identity, report, spec};
use crate::gateway::GatewayMock;
use crate::maintenance::MaintenanceMock;
use crate::runtime_fakes::{IncomingConnection, WaitingAgent, WaitingOperation};
use crate::timed_fakes::ManualDelay;

use super::{AgentDaemon, Daemon};

#[tokio::test]
async fn daemon_reports_every_wait_beyond_queue_capacity_while_control_stays_pending() {
    let snapshot = report().await;
    let reports = Arc::new(AtomicUsize::new(0));
    let observed = reports.clone();
    let finished = Arc::new(AtomicBool::new(false));
    let is_finished = finished.clone();
    let removing = finished.clone();
    let (gateway, mut gateway_ready) = mpsc::unbounded_channel();
    let (reconciled, mut reconcile_ready) = mpsc::unbounded_channel();
    let reconciliations = Arc::new(AtomicUsize::new(0));
    let reconciled_count = reconciliations.clone();
    let mock = Arc::new(Unimock::new((
        AgentMock::control_connection
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        AgentMock::restore.some_call(matching!()).returns(Ok(())),
        AgentMock::complete_removal
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| Ok(is_finished.load(Ordering::SeqCst)))),
        AgentMock::remove
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                removing.store(true, Ordering::SeqCst);
                Ok(())
            })),
        AgentMock::control
            .some_call(matching!(ControlMessage::StartRunner { .. }))
            .returns(Ok(None)),
        AgentMock::report
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(snapshot.clone()))),
        AgentMock::reconcile
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                reconciled_count.fetch_add(1, Ordering::SeqCst);
                reconciled.send(()).unwrap();
                Ok(())
            })),
        GatewayMock::reconnect
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                gateway.send(()).unwrap();
                Ok(())
            })),
        MaintenanceMock::run.some_call(matching!()).returns(()),
        DelayMock::wait.each_call(matching!(10)).returns(()),
        ConnectionMock::receive
            .some_call(matching!())
            .returns(Ok(ControlEvent::Message(Box::new(
                ControlMessage::StartRunner {
                    command_id: CommandId("blocked".to_owned()),
                    runner: spec(),
                },
            )))),
        ConnectionMock::send
            .each_call(matching!((message) if matches!(message, AgentMessage::Report { .. })))
            .answers_arc(Arc::new(move |_, _| {
                observed.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
    )));
    let (release, barrier) = oneshot::channel();
    let completed = Arc::new(AtomicBool::new(false));
    let (started, mut command_started) = mpsc::unbounded_channel();
    let agent = Arc::new(WaitingAgent {
        operation: WaitingOperation::Control,
        delegate: mock.clone(),
        started,
        barrier: Mutex::new(Some(barrier)),
        completed: completed.clone(),
    });
    let (permit, receive) = mpsc::unbounded_channel();
    permit.send(()).unwrap();
    let connection = Arc::new(IncomingConnection {
        delegate: mock.clone(),
        permit: Mutex::new(receive),
    });
    let (requests, mut waits) = mpsc::unbounded_channel();
    let daemon = AgentDaemon {
        agent,
        connection,
        gateway: mock.clone(),
        maintenance: mock.clone(),
        report_delay: Arc::new(ManualDelay {
            delegate: mock.clone(),
            requests,
        }),
    };
    let (_guests, guest_events) = mpsc::channel(1);
    let (remove, removal) = mpsc::channel(1);
    let run = unconstrained(daemon.run(Some(identity()), guest_events, false, removal));
    pin_mut!(run);
    assert!(matches!(poll!(run.as_mut()), Poll::Pending));
    gateway_ready.recv().await.unwrap();
    assert!(matches!(poll!(run.as_mut()), Poll::Pending));
    command_started.recv().await.unwrap();
    for interval in 1..=130 {
        let wait = waits.try_recv().unwrap();
        assert_eq!(wait.seconds, 10);
        assert_eq!(
            wait.release.send(()),
            Ok(()),
            "unrelated events must not cancel the report wait"
        );
        assert!(matches!(poll!(run.as_mut()), Poll::Pending));
        assert_eq!(
            reports.load(Ordering::SeqCst),
            interval,
            "interval {interval}"
        );
        assert!(!completed.load(Ordering::SeqCst));
    }
    release.send(()).unwrap();
    reconcile_ready.recv().await.unwrap();
    assert_eq!(reconciliations.load(Ordering::SeqCst), 1);
    remove.send(()).await.unwrap();
    run.await.unwrap();
    assert_eq!(reports.load(Ordering::SeqCst), 130);
}

#[tokio::test]
async fn daemon_removes_a_rejected_connected_machine_without_another_handshake() {
    let finished = Arc::new(AtomicBool::new(false));
    let is_finished = finished.clone();
    let removals = Arc::new(AtomicUsize::new(0));
    let removed = removals.clone();
    let reconnections = Arc::new(AtomicUsize::new(0));
    let reconnected = reconnections.clone();
    let (gateway, mut gateway_ready) = mpsc::unbounded_channel();
    let (maintenance, mut maintenance_ready) = mpsc::unbounded_channel();
    let mock = Arc::new(Unimock::new((
        AgentMock::control_connection
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        AgentMock::restore.some_call(matching!()).returns(Ok(())),
        AgentMock::complete_removal
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| Ok(is_finished.load(Ordering::SeqCst)))),
        AgentMock::remove
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                removed.fetch_add(1, Ordering::SeqCst);
                finished.store(true, Ordering::SeqCst);
                Ok(())
            })),
        GatewayMock::reconnect
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                reconnected.fetch_add(1, Ordering::SeqCst);
                gateway.send(()).unwrap();
                Ok(())
            })),
        MaintenanceMock::run
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                maintenance.send(()).unwrap();
            })),
        DelayMock::wait.each_call(matching!(10)).returns(()),
        ConnectionMock::receive
            .some_call(matching!())
            .returns(Ok(ControlEvent::CredentialRejected)),
    )));
    let (permit, receive) = mpsc::unbounded_channel();
    permit.send(()).unwrap();
    let (requests, _waits) = mpsc::unbounded_channel();
    let daemon = AgentDaemon {
        agent: mock.clone(),
        connection: Arc::new(IncomingConnection {
            delegate: mock.clone(),
            permit: Mutex::new(receive),
        }),
        gateway: mock.clone(),
        maintenance: mock.clone(),
        report_delay: Arc::new(ManualDelay {
            delegate: mock.clone(),
            requests,
        }),
    };
    let (_guests, guest_events) = mpsc::channel(1);
    let (_remove, removal) = mpsc::channel(1);
    let run = unconstrained(daemon.run(Some(identity()), guest_events, false, removal));
    pin_mut!(run);
    assert!(matches!(poll!(run.as_mut()), Poll::Pending));
    gateway_ready.recv().await.unwrap();
    maintenance_ready.recv().await.unwrap();
    let result = poll!(run.as_mut());
    assert_eq!(removals.load(Ordering::SeqCst), 1);
    assert_eq!(reconnections.load(Ordering::SeqCst), 1);
    match result {
        Poll::Ready(result) => result,
        Poll::Pending => run.await,
    }
    .unwrap();
}
