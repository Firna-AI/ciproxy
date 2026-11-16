//! Final remote removal forgets the credential and every control-plane route.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::application_endpoint::{ConfiguredAction, configure};
use crate::boundaries::{CacheMock, GuestProvenance, Store, StoreMock, VmsMock};
use crate::cli::Action;
use crate::controller::Agent;
use crate::endpoint::{CommandEndpoint, EndpointFilesMock};
use crate::fixtures::{controller, identity, vm};
use crate::identity_store::IdentityStore;
use crate::state::{MachineState, Mode};
use crate::terminal_completion::CompletionAction;

#[tokio::test]
async fn guest_exit_discards_vm_and_removal_finishes_only_after_the_job() {
    let mock = Unimock::new((
        VmsMock::networks
            .some_call(matching!())
            .returns(Ok(Vec::new())),
        VmsMock::remove_network
            .some_call(matching!(1))
            .returns(Ok(())),
        VmsMock::stop
            .some_call(matching!((record) if record.report.runner_id.0 == "job"))
            .returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        StoreMock::save
            .each_call(
                matching!((state) if state.runners.is_empty() && state.mode == Mode::Removed),
            )
            .answers(&|_, _| Ok(())),
    ));
    let agent = controller(
        MachineState {
            identity: Some(identity()),
            runners: vec![vm("job", true)],
            mode: Mode::Removed,
            ..MachineState::default()
        },
        mock,
    );
    assert_eq!(
        agent
            .guest(
                GuestProvenance::InitialConfiguration,
                "job",
                GuestToAgent::RunnerExited {
                    runner_id: RunnerId("job".to_owned()),
                    exit: RunnerExit { code: Some(0) }
                }
            )
            .await
            .unwrap(),
        Some(AgentMessage::RunnerExited {
            runner_id: RunnerId("job".to_owned()),
            exit: RunnerExit { code: Some(0) }
        })
    );
    assert_eq!(agent.state.lock().await.pending_networks, [1].into());
    assert!(
        agent
            .complete_removal(CompletionAction::Stop)
            .await
            .unwrap()
    );
    let state = agent.state.lock().await.clone();
    assert_eq!(state.identity, None);
    assert!(
        !serde_json::to_string(&state)
            .unwrap()
            .contains("gateway.test")
    );
    let legacy = Arc::new(Mutex::new(Some(
        "wss://retired-preview.ciproxy.example/agent/".to_owned(),
    )));
    let read = legacy.clone();
    let remove = legacy.clone();
    let mock = Arc::new(
        Unimock::new((
            StoreMock::load.some_call(matching!()).returns(Ok(state)),
            EndpointFilesMock::read
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| Ok(read.lock().unwrap().clone())))
                .at_least_times(0),
            EndpointFilesMock::remove
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    *remove.lock().unwrap() = None;
                    Ok(())
                }))
                .at_least_times(0),
            EndpointFilesMock::environment
                .some_call(matching!())
                .answers(&|_| Ok(None))
                .at_least_times(0),
        ))
        .no_verify_in_drop(),
    );
    let store = IdentityStore {
        store: mock.clone(),
        endpoints: mock.clone(),
    };
    let settings = CommandEndpoint { files: mock };
    let state = store.load().unwrap();
    assert_eq!(state.identity, None);
    let action = configure(
        Action::Install {
            token: Secret::new("synthetic-production-token"),
            agent_url: None,
        },
        &settings,
    )
    .unwrap();
    let ConfiguredAction::Install { endpoint, .. } = action else {
        panic!("removed machine must accept a fresh install");
    };
    assert_eq!(endpoint.as_str(), "wss://app.ciproxy.dev/agent/");
    assert_eq!(*legacy.lock().unwrap(), None);
}
