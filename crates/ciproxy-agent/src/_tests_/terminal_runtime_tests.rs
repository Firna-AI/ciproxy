//! Daemon replies and reports progress while checked completion retains identity.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::runners::RunnerExit;
use futures_util::{pin_mut, poll};
use tokio::sync::{self, mpsc};
use tokio::task::unconstrained;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, ConnectionMock, DelayMock, GuestEvent, GuestProvenance};
use crate::cache_authority::CacheAuthority;
use crate::fixtures::{controller, identity, report_dependencies, vm};
use crate::gateway::GatewayMock;
use crate::maintenance::MaintenanceMock;
use crate::network_barriers::HeldDeletion;
use crate::network_fixtures::prepared_network;
use crate::network_fixtures::{HostNetworks, commands, restored, runtime_with_network, saving};
use crate::process::{CommandsMock, UnitState};
use crate::runtime::{AgentDaemon, Daemon};
use crate::runtime_fakes::IncomingConnection;
use crate::state::{MachineState, Mode};
use crate::timed_fakes::ManualDelay;
use crate::vm_files::VmFilesMock;
use crate::vsock::GuestsMock;

#[tokio::test]
async fn daemon_sends_exit_and_reports_while_terminal_cleanup_waits_then_finalizes_identity() {
    let host = Arc::new(Mutex::new(HostNetworks {
        namespaces: ["cpx1".to_owned()].into(),
        ..HostNetworks::default()
    }));
    let mut runner = vm("runner", true);
    runner.cache_authority = CacheAuthority::GitHubOnly;
    let initial = MachineState {
        mode: Mode::Removed,
        identity: Some(identity()),
        runners: vec![runner],
        ..MachineState::default()
    };
    let saved = Arc::new(Mutex::new(serde_json::to_vec(&initial).unwrap()));
    let messages = Arc::new(Mutex::new(Vec::new()));
    let recorded = messages.clone();
    let (sent, mut sent_ready) = mpsc::unbounded_channel();
    let (gateway, mut gateway_ready) = mpsc::unbounded_channel();
    let mock = Arc::new(Unimock::new((
        commands(host.clone()),
        saving(saved.clone()),
        report_dependencies(),
        GuestsMock::attach
            .some_call(matching!(_, None))
            .returns(Ok(())),
        GuestsMock::detach.some_call(matching!(_)).returns(Ok(())),
        GuestsMock::send
            .some_call(matching!(_, _))
            .answers(&|_, _, _| Ok(()))
            .at_least_times(0),
        CommandsMock::unit
            .some_call(matching!(_))
            .returns(Ok(UnitState::Absent)),
        VmFilesMock::remove.some_call(matching!(_)).returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        CacheMock::refresh_budget
            .each_call(matching!(_))
            .answers(&|_, _| Ok(()))
            .at_least_times(0),
        GatewayMock::reconnect
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                gateway.send(()).unwrap();
                Ok(())
            })),
        MaintenanceMock::run.some_call(matching!()).returns(()),
        DelayMock::wait.each_call(matching!(10)).returns(()),
        ConnectionMock::send
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, message| {
                recorded.lock().unwrap().push(message.clone());
                sent.send(()).unwrap();
                Ok(())
            })),
    )));
    let (started, cleanup_ready) = sync::oneshot::channel();
    let (release, barrier) = sync::oneshot::channel();
    let vms = runtime_with_network(
        mock.clone(),
        Arc::new(prepared_network(Arc::new(HeldDeletion {
            delegate: mock.clone(),
            inspection_started: sync::Mutex::new(None),
            inspection_release: sync::Mutex::new(None),
            started: sync::Mutex::new(Some(started)),
            release: sync::Mutex::new(Some(barrier)),
        }))),
    );
    let mut agent = controller(initial, mock.as_ref().clone());
    agent.vms = Arc::new(vms);
    let agent = Arc::new(agent);
    let (_permit, receive) = mpsc::unbounded_channel();
    let (requests, mut waits) = mpsc::unbounded_channel();
    let daemon = AgentDaemon {
        agent: agent.clone(),
        connection: Arc::new(IncomingConnection {
            delegate: mock.clone(),
            permit: sync::Mutex::new(receive),
        }),
        gateway: mock.clone(),
        maintenance: mock.clone(),
        report_delay: Arc::new(ManualDelay {
            delegate: mock.clone(),
            requests,
        }),
    };
    let (guests, guest_events) = mpsc::channel(1);
    let (_remove, removal) = mpsc::channel(1);
    let run = unconstrained(daemon.run(Some(identity()), guest_events, false, removal));
    pin_mut!(run);
    assert!(poll!(run.as_mut()).is_pending());
    gateway_ready.recv().await.unwrap();
    assert!(poll!(run.as_mut()).is_pending());
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
    assert!(poll!(run.as_mut()).is_pending());
    tokio::select! {
        result = run.as_mut() => panic!("daemon returned before the exit reply: {result:?}"),
        sent = sent_ready.recv() => sent.unwrap(),
    }
    assert_eq!(
        *messages.lock().unwrap(),
        vec![AgentMessage::RunnerExited {
            runner_id: RunnerId("runner".to_owned()),
            exit: RunnerExit { code: Some(0) },
        }]
    );
    let wait = waits.recv().await.unwrap();
    assert_eq!(wait.seconds, 10);
    wait.release.send(()).unwrap();
    assert!(poll!(run.as_mut()).is_pending());
    cleanup_ready.await.unwrap();
    tokio::select! {
        result = run.as_mut() => panic!("daemon returned while network deletion waits: {result:?}"),
        sent = sent_ready.recv() => sent.unwrap(),
    }
    assert_eq!(messages.lock().unwrap().len(), 2);
    assert!(
        matches!(&messages.lock().unwrap()[1], AgentMessage::Report { report } if report.runners.is_empty())
    );
    assert_eq!(restored(&saved).identity, Some(identity()));
    assert_eq!(restored(&saved).pending_networks, [1].into());
    release.send(()).unwrap();
    run.await.unwrap();
    assert_eq!(restored(&saved).identity, None);
    assert_eq!(restored(&saved).mode, Mode::Removed);
    assert_eq!(restored(&saved).pending_networks, Default::default());
    assert_eq!(host.lock().unwrap().namespaces, Default::default());
}
