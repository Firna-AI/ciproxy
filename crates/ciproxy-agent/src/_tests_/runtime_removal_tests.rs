//! Worker cleanup completion wakes a disconnected daemon without external input.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Poll;

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::runners::RunnerExit;
use futures_util::{pin_mut, poll};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::unconstrained;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{DelayMock, GuestEvent, GuestProvenance};
use crate::controller::AgentMock;
use crate::fixtures::identity;
use crate::maintenance::MaintenanceMock;
use crate::runtime_fakes::{WaitingAgent, WaitingOperation};
use crate::timed_fakes::ManualDelay;

use super::{AgentDaemon, Daemon};

#[tokio::test]
async fn disconnected_removal_exits_after_cleanup_without_another_input_event() {
    let finished = Arc::new(AtomicBool::new(false));
    let observed = finished.clone();
    let cleaned = finished.clone();
    let (exited, mut exit_ready) = mpsc::unbounded_channel();
    let (reconciled, mut cleanup_ready) = mpsc::unbounded_channel();
    let (checked, mut terminal_checked) = mpsc::unbounded_channel();
    let mock = Arc::new(Unimock::new((
        AgentMock::restore.some_call(matching!()).returns(Ok(())),
        AgentMock::control_connection
            .some_call(matching!(false))
            .returns(Ok(())),
        AgentMock::finished
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let finished = observed.load(Ordering::SeqCst);
                if finished {
                    checked.send(()).unwrap();
                }
                finished
            })),
        AgentMock::guest
            .some_call(matching!(_, "runner", GuestToAgent::RunnerExited { .. }))
            .answers_arc(Arc::new(move |_, _, _, _| {
                exited.send(()).unwrap();
                Ok(Some(AgentMessage::RunnerExited {
                    runner_id: RunnerId("runner".to_owned()),
                    exit: RunnerExit { code: Some(0) },
                }))
            })),
        AgentMock::reconcile
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                cleaned.store(true, Ordering::SeqCst);
                reconciled.send(()).unwrap();
                Ok(())
            })),
        MaintenanceMock::run.some_call(matching!()).returns(()),
        DelayMock::wait.each_call(matching!(10)).returns(()),
    )));
    let (started, mut reconciliation_started) = mpsc::unbounded_channel();
    let (release, barrier) = oneshot::channel();
    let completed = Arc::new(AtomicBool::new(false));
    let agent = Arc::new(WaitingAgent {
        operation: WaitingOperation::Reconcile,
        delegate: mock.clone(),
        started,
        barrier: Mutex::new(Some(barrier)),
        completed: completed.clone(),
    });
    let (requests, mut waits) = mpsc::unbounded_channel();
    let daemon = AgentDaemon {
        agent,
        connection: mock.clone(),
        gateway: mock.clone(),
        maintenance: mock.clone(),
        report_delay: Arc::new(ManualDelay {
            delegate: mock.clone(),
            requests,
        }),
    };
    let (guests, guest_events) = mpsc::channel(1);
    guests
        .send(GuestEvent {
            provenance: GuestProvenance::Recovery,
            configuration_lost: false,
            runner_id: RunnerId("runner".to_owned()),
            message: GuestToAgent::RunnerExited {
                runner_id: RunnerId("runner".to_owned()),
                exit: RunnerExit { code: Some(0) },
            },
        })
        .await
        .unwrap();
    let (_remove, removal) = mpsc::channel(1);
    let run = unconstrained(daemon.run(identity(), guest_events, true, removal));
    pin_mut!(run);
    assert!(poll!(run.as_mut()).is_pending());
    exit_ready.recv().await.unwrap();
    assert!(poll!(run.as_mut()).is_pending());
    let wait = waits.recv().await.unwrap();
    assert_eq!(wait.seconds, 10);
    wait.release.send(()).unwrap();
    assert!(poll!(run.as_mut()).is_pending());
    reconciliation_started.recv().await.unwrap();
    assert!(poll!(run.as_mut()).is_pending());
    release.send(()).unwrap();
    cleanup_ready.recv().await.unwrap();
    assert!(completed.load(Ordering::SeqCst));
    let result = poll!(run.as_mut());
    assert_eq!(
        terminal_checked.try_recv(),
        Ok(()),
        "completed cleanup must wake the disconnected daemon"
    );
    match result {
        Poll::Ready(result) => result,
        Poll::Pending => run.await,
    }
    .unwrap();
}
