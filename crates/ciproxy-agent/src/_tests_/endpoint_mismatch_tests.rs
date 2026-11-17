//! A repair with another command endpoint preserves the complete saved identity.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::application_endpoint::{ConfiguredAction, configure};
use crate::boundaries::{ConnectionMock, HostMock, ServiceMock, StoreMock};
use crate::cli::Action;
use crate::endpoint::{CommandEndpoint, EndpointFilesMock};
use crate::fixtures::identity;
use crate::host_preparation::PreparationMock;
use crate::installation::{Install, Installer};
use crate::state::MachineState;

async fn repair(agent_url: Option<&str>) -> (String, MachineState, Vec<AgentMessage>) {
    let mut identity = identity();
    identity.endpoint = Some("wss://preview.ciproxy.example/agent/".parse().unwrap());
    let initial = MachineState {
        identity: Some(identity),
        ..MachineState::default()
    };
    let saved = Arc::new(Mutex::new(initial.clone()));
    let writes = saved.clone();
    let sent = Arc::new(Mutex::new(Vec::new()));
    let messages = sent.clone();
    let connected = Arc::new(Mutex::new(0));
    let connections = connected.clone();
    let installed = Arc::new(Mutex::new(0));
    let services = installed.clone();
    let loads = Arc::new(Mutex::new(0));
    let loaded = loads.clone();
    let saved_count = Arc::new(Mutex::new(0));
    let written = saved_count.clone();
    let store = Arc::new(
        Unimock::new((
            StoreMock::load
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    *loaded.lock().unwrap() += 1;
                    Ok(initial.clone())
                })),
            StoreMock::save
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, state| {
                    *written.lock().unwrap() += 1;
                    *writes.lock().unwrap() = state.clone();
                    Ok(())
                })),
        ))
        .no_verify_in_drop(),
    );
    let connection = Arc::new(
        Unimock::new((
            ConnectionMock::connect
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    *connections.lock().unwrap() += 1;
                    Ok(())
                })),
            ConnectionMock::send
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, message| {
                    messages.lock().unwrap().push(message.clone());
                    Ok(())
                })),
        ))
        .no_verify_in_drop(),
    );
    let mock = Arc::new(Unimock::new((
        HostMock::require_root
            .some_call(matching!())
            .returns(Ok(())),
        PreparationMock::prepare
            .some_call(matching!())
            .returns(Ok(())),
        ServiceMock::install
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                *services.lock().unwrap() += 1;
                Ok(())
            })),
    )));
    let environment = if agent_url.is_none() {
        Unimock::new(
            EndpointFilesMock::environment
                .some_call(matching!())
                .returns(Ok(None)),
        )
    } else {
        Unimock::new(())
    };
    let settings = CommandEndpoint {
        files: Arc::new(environment),
    };
    let action = configure(
        Action::Install {
            token: Secret::new("synthetic-command-token"),
            agent_url: agent_url.map(|value| value.parse().unwrap()),
        },
        &settings,
    )
    .unwrap();
    let ConfiguredAction::Install { token, endpoint } = action else {
        panic!("repair must keep the install action");
    };
    let installer = Installer {
        endpoint,
        store,
        host: mock.clone(),
        service: mock.clone(),
        connection,
        agent: mock.clone(),
        preparation: mock,
    };
    let message = installer.install(token).await.unwrap().message();
    assert_eq!(*loads.lock().unwrap(), 1);
    assert_eq!(*saved_count.lock().unwrap(), 0);
    assert_eq!(*connected.lock().unwrap(), 0);
    assert_eq!(*installed.lock().unwrap(), 1);
    let state = saved.lock().unwrap().clone();
    let messages = sent.lock().unwrap().clone();
    (message, state, messages)
}

#[tokio::test]
async fn different_agent_url_keeps_identity_and_sends_nothing_to_the_new_endpoint() {
    let (message, state, sent) = repair(Some("wss://other.ciproxy.example/agent/")).await;
    let mut expected = identity();
    expected.endpoint = Some("wss://preview.ciproxy.example/agent/".parse().unwrap());
    assert_eq!(state.identity, Some(expected));
    assert_eq!(sent, Vec::new());
    assert_eq!(
        message,
        "ciproxy agent repaired. Machine stays at wss://preview.ciproxy.example/agent/. Command selected wss://other.ciproxy.example/agent/. To move the machine, uninstall first."
    );
}

#[tokio::test]
async fn production_command_keeps_preview_and_last_line_says_to_uninstall_first() {
    let (message, state, sent) = repair(None).await;
    let mut expected = identity();
    expected.endpoint = Some("wss://preview.ciproxy.example/agent/".parse().unwrap());
    assert_eq!(state.identity, Some(expected));
    assert_eq!(sent, Vec::new());
    assert_eq!(
        message.lines().last(),
        Some(
            "ciproxy agent repaired. Machine stays at wss://preview.ciproxy.example/agent/. Command selected wss://app.ciproxy.dev/agent/. To move the machine, uninstall first."
        )
    );
}
