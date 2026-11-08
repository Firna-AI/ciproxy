//! Removed startup drains saved jobs without a credential or gateway access.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::runners::RunnerExit;
use futures_util::{pin_mut, poll};
use tokio::sync::mpsc;
use tokio::task::unconstrained;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, DelayMock, GuestEvent, GuestProvenance, ImagesMock, VmsMock};
use crate::cache_authority::CacheAuthority;
use crate::error::Error;
use crate::fixtures::{controller, vm};
use crate::host_preparation::PreparationMock;
use crate::maintenance::MaintenanceMock;
use crate::network_fixtures::{restored, saving};
use crate::runtime::{AgentDaemon, Daemon};
use crate::state::{MachineState, Mode};
use crate::timed_fakes::ManualDelay;

use super::{AgentStartup, Startup, StartupState};

#[tokio::test]
async fn removed_startup_drains_saved_jobs_without_identity_or_gateway_access() {
    let mut runner = vm("runner", true);
    runner.cache_authority = CacheAuthority::GitHubOnly;
    let initial = MachineState {
        runners: vec![runner.clone()],
        ..MachineState::default()
    };
    let saved = Arc::new(Mutex::new(serde_json::to_vec(&initial).unwrap()));
    let mock = Arc::new(Unimock::new((
        saving(saved.clone()),
        PreparationMock::verify
            .each_call(matching!())
            .answers(&|_| Ok(())),
        ImagesMock::recover
            .each_call(matching!())
            .answers(&|_| Ok(())),
        VmsMock::restore.some_call(matching!(_)).returns(Ok(())),
        VmsMock::send.some_call(matching!(_, _)).returns(Ok(())),
        VmsMock::stop.some_call(matching!(_)).returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        VmsMock::networks
            .some_call(matching!())
            .returns(Ok(vec![1])),
        VmsMock::remove_network
            .some_call(matching!(1))
            .returns(Ok(())),
        MaintenanceMock::run.some_call(matching!()).returns(()),
        DelayMock::wait.each_call(matching!(10)).returns(()),
    )));
    let agent = Arc::new(controller(initial, mock.as_ref().clone()));
    let startup = AgentStartup {
        agent: agent.clone(),
        preparation: mock.clone(),
        images: mock.clone(),
    };
    assert!(matches!(
        startup.prepare(None, false).await,
        Err(Error::NotInstalled)
    ));
    assert_eq!(agent.state.lock().await.runners, vec![runner.clone()]);
    agent.state.lock().await.mode = Mode::Removed;
    let result = startup.prepare(None, true).await;
    assert!(
        matches!(&result, Ok(StartupState::Run(None))),
        "Removed startup must preserve jobs without requiring identity: {result:?}"
    );
    let StartupState::Run(identity) = result.unwrap() else {
        panic!("running jobs must reach the daemon");
    };
    let (requests, _waits) = mpsc::unbounded_channel();
    let daemon = AgentDaemon {
        agent: agent.clone(),
        connection: mock.clone(),
        gateway: mock.clone(),
        maintenance: mock.clone(),
        report_delay: Arc::new(ManualDelay {
            delegate: mock.clone(),
            requests,
        }),
    };
    let (guests, guest_events) = mpsc::channel(1);
    let (_remove, removal) = mpsc::channel(1);
    let run = unconstrained(daemon.run(identity, guest_events, true, removal));
    pin_mut!(run);
    assert!(
        poll!(run.as_mut()).is_pending(),
        "Removed daemon must wait for its saved job"
    );
    assert_eq!(agent.state.lock().await.runners, vec![runner]);
    assert_eq!(agent.state.lock().await.mode, Mode::Removed);
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
    run.await.unwrap();
    let state = restored(&saved);
    assert_eq!(state.mode, Mode::Removed);
    assert_eq!(state.identity, None);
    assert_eq!(state.runners, vec![]);
    assert_eq!(state.pending_networks, Default::default());
}
