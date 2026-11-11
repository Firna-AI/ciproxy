//! Host cleanup waits never hold report state or restore a nonterminal mode.

use std::sync::{Arc, Mutex};
use std::task::Poll;

use futures_util::{pin_mut, poll};
use tokio::sync;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::CacheMock;
use crate::controller::Agent;
use crate::fixtures::{controller, report_dependencies};
use crate::network_barriers::HeldDeletion;
use crate::network_fixtures::prepared_network;
use crate::network_fixtures::{HostNetworks, commands, restored, runtime_with_network, saving};
use crate::state::{MachineState, Mode};
use crate::terminal_completion::CompletionAction;

#[tokio::test]
async fn reports_and_terminal_removal_progress_while_network_deletion_waits() {
    let host = Arc::new(Mutex::new(HostNetworks {
        namespaces: ["cpx1".to_owned()].into(),
        ..HostNetworks::default()
    }));
    let saved = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        commands(host.clone()),
        saving(saved.clone()),
        report_dependencies(),
        CacheMock::refresh_budget
            .some_call(matching!(_))
            .returns(Ok(())),
    )));
    let (started, ready) = sync::oneshot::channel();
    let (release, barrier) = sync::oneshot::channel();
    let (inspection_started, inspection_ready) = sync::oneshot::channel();
    let (inspection_release, inspection_barrier) = sync::oneshot::channel();
    let vms = runtime_with_network(
        mock.clone(),
        Arc::new(prepared_network(Arc::new(HeldDeletion {
            delegate: mock.clone(),
            inspection_started: sync::Mutex::new(Some(inspection_started)),
            inspection_release: sync::Mutex::new(Some(inspection_barrier)),
            started: sync::Mutex::new(Some(started)),
            release: sync::Mutex::new(Some(barrier)),
        }))),
    );
    let mut agent = controller(
        MachineState {
            mode: Mode::Draining,
            ..MachineState::default()
        },
        mock.as_ref().clone(),
    );
    agent.vms = Arc::new(vms);
    let startup = agent.restore();
    pin_mut!(startup);
    assert!(matches!(poll!(startup.as_mut()), Poll::Ready(Ok(()))));
    assert_eq!(host.lock().unwrap().deletions.len(), 0);
    let reconcile = agent.reconcile();
    pin_mut!(reconcile);
    assert!(poll!(reconcile.as_mut()).is_pending());
    inspection_ready.await.unwrap();
    let report = agent.report();
    pin_mut!(report);
    let report = match poll!(report.as_mut()) {
        Poll::Ready(Ok(report)) => report,
        other => panic!("reports must continue while cleanup waits: {other:?}"),
    };
    assert_eq!(report.runners.len(), 0);
    assert_eq!(report.slots.len(), 6);
    agent.remove().await.unwrap();
    assert_eq!(agent.state.lock().await.mode, Mode::Removed);
    assert_eq!(
        agent.state.lock().await.pending_networks,
        Default::default()
    );
    let complete = agent.complete_removal(CompletionAction::Stop);
    pin_mut!(complete);
    assert!(poll!(complete.as_mut()).is_pending());
    inspection_release.send(()).unwrap();
    assert!(poll!(reconcile.as_mut()).is_pending());
    ready.await.unwrap();
    let report = agent.report();
    pin_mut!(report);
    assert!(matches!(poll!(report.as_mut()), Poll::Ready(Ok(_))));
    assert_eq!(agent.state.lock().await.pending_networks, [1].into());
    assert!(poll!(complete.as_mut()).is_pending());
    release.send(()).unwrap();
    reconcile.await.unwrap();
    assert_eq!(restored(&saved).mode, Mode::Removed);
    assert!(complete.await.unwrap());
}
